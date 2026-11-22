"""Private GTK stock ownership A/B control, using the unchanged nine lifecycle gates."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from lifecycle import ROOT
from gtk_egl_isolated import provenance, require_loader
from gtk_egl_source_test import PATCHED_SHA256

STOCK_PATCH = ROOT/'docs/performance/gtk-4.24.1-stock-egl-ownership.patch'
STOCK_SOURCE_SHA256 = '5a5591e2e72b2b93f49174cd25e3e975a9022d73e9e84990af93eb47ea6a9f04'
STOCK_PATCHED_SHA256 = '5aaf601a1dd892ba08f34832ff9751eecc4ca5c0304efd14f025f61c11fa1f3c'


def stock_provenance(build, expected):
    meta = provenance(build, PATCHED_SHA256)
    sink = Path(meta['source'])/'gtk/media/gtkgstsink.c'
    if hashlib.sha256(sink.read_bytes()).hexdigest() != expected:
        raise RuntimeError('stock sink source does not match reviewed ownership policy')
    # GTK 4.24 links this backend into libgtk; the old modules/media filename
    # does not establish availability. Never substitute GtkNoMediaFile.
    config = (build/'config.h').read_text()
    if '#define HAVE_GSTREAMER 1' not in config.splitlines():
        raise RuntimeError('real built-in GTK GStreamer backend is required')
    meta.update(stock_sink_sha256=expected, stock_backend='built-in HAVE_GSTREAMER')
    return meta


def compare_sources(control, candidate):
    def sources(root):
        return {p.relative_to(root) for directory in ['gdk', 'gsk', 'gtk', 'modules']
                for p in (root/directory).rglob('*')
                if p.suffix in ('.c', '.h', '.glsl', '.frag', '.vert') and p.is_file()}
    files = sources(control)
    if files != sources(candidate):
        raise RuntimeError('unreviewed GTK source addition/removal')
    for relative in files:
        if str(relative) != 'gtk/media/gtkgstsink.c' and (control/relative).read_bytes() != (candidate/relative).read_bytes():
            raise RuntimeError('unreviewed GTK change: '+str(relative))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--control-build', type=Path, required=True)
    parser.add_argument('--candidate-build', type=Path, required=True)
    parser.add_argument('--session', choices=['native', 'x11', 'sway'], default='native')
    parser.add_argument('--software', action='store_true')
    args = parser.parse_args()
    for key in ('LD_PRELOAD', 'GTK_MEDIA', 'PIVRA_VIDEO_BACKEND', 'PIVRA_VIDEO_TEST_MODULE_DIR'):
        if os.environ.get(key):
            parser.error('unset '+key+' for actual stock/GPU comparison')
    if not args.software and os.environ.get('PIVRA_GPU_ALLOW_SOFTWARE_TEST') == '1':
        parser.error('software permission requires --software')
    out = Path(tempfile.mkdtemp(prefix='pivra-gtk-stock-lifecycle-'))
    print('Private stock lifecycle comparison:', out, flush=True)
    variants = {'control': stock_provenance(args.control_build, STOCK_SOURCE_SHA256),
                'candidate': stock_provenance(args.candidate_build, STOCK_PATCHED_SHA256)}
    compare_sources(*(Path(variants[k]['source']) for k in ['control', 'candidate']))
    # Public headers are identical. Compile one application/adapter binary for
    # both variants; only the runtime GTK library changes in the A/B run.
    control = variants['control']
    common = dict(os.environ,
                  PKG_CONFIG_PATH=str(Path(control['build'])/'meson-uninstalled')+':'+os.environ.get('PKG_CONFIG_PATH', ''),
                  LDFLAGS='-L'+str(Path(control['library']).parent)+' '+os.environ.get('LDFLAGS', ''))
    subprocess.run(['cargo', 'build', '--locked', '--example', 'integrated_video_probe', '--example', 'video_color_probe'], cwd=ROOT, env=common, check=True)
    target = Path(json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--no-deps', '--format-version', '1'], cwd=ROOT, env=common))['target_directory'])/'debug/examples/integrated_video_probe'
    binary_sha256 = hashlib.sha256(target.read_bytes()).hexdigest()
    results = {}
    for label, meta in variants.items():
        env = dict(common, LD_LIBRARY_PATH=str(Path(meta['library']).parent)+':'+os.environ.get('LD_LIBRARY_PATH', ''))
        env.pop('GTK_PATH', None)
        env.pop('GIO_EXTRA_MODULES', None)
        # ldd plus the mixed probe's actual /proc/self/maps hashes establish the
        # selected library; a missing real stock backend still exits 77.
        (out/(label+'-ldd.log')).write_text(require_loader(target, meta['library'], env))
        command = [sys.executable, '-B', str(ROOT/'tests/video_gpu/lifecycle.py'), '--session', args.session]
        if args.software:
            command.append('--software')
        with (out/(label+'.log')).open('w') as log:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        text = (out/(label+'.log')).read_text(errors='replace')
        print(text, end='', flush=True)
        reports = [Path(line.removeprefix('Logs: '))/'report.json' for line in text.splitlines() if line.startswith('Logs: ')]
        report = json.loads(reports[0].read_text()) if len(reports)==1 and reports[0].exists() else None
        rows = report['results'] if report else []
        mixed = next((r for r in rows if r['name']=='mixed-lifecycle'), None)
        verified = bool(mixed and mixed.get('libraries_sha256', {}).get(meta['library'])==meta['sha256'])
        same_binary = hashlib.sha256(target.read_bytes()).hexdigest()==binary_sha256
        passed = result.returncode==0 and same_binary and len(rows)==9 and all(r['status']=='PASS' for r in rows) and verified
        results[label] = dict(exit=result.returncode, same_application_binary=same_binary, application_sha256=binary_sha256, all_nine_verified=passed, runtime_library_verified=verified,
                              report=str(reports[0]) if reports else None, mixed=mixed)
    (out/'report.json').write_text(json.dumps(dict(variants=variants, results=results, software_control=args.software,
        stock_patch_sha256=hashlib.sha256(STOCK_PATCH.read_bytes()).hexdigest(),
        native_AMD_acceptance='PENDING user hardware result' if args.software else 'requires explicit hardware identity/visual report'), indent=2)+'\n')
    print('CONTROL', results['control']['exit'], '; CANDIDATE', 'PASS 9/9' if results['candidate']['all_nine_verified'] else 'FAIL/NOT RUN', flush=True)
    return 0 if results['candidate']['all_nine_verified'] else 77 if results['candidate']['exit']==77 else 1


if __name__ == '__main__':
    raise SystemExit(main())
