#!/usr/bin/env python3
"""Build and launch the file-scoped GPU prototype with stock GTK (no install)."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import tempfile

from build import ROOT, build


def eligible(stream):
    return (stream.get('codec_type') == 'video' and
            stream.get('color_primaries') == 'bt709' and
            stream.get('color_transfer') == 'bt709' and
            stream.get('color_space') == 'bt709' and
            stream.get('color_range') in ('tv', 'pc') and
            stream.get('pix_fmt') in ('yuv420p', 'yuvj420p', 'yuv422p', 'yuv444p'))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('video', type=Path)
    p.add_argument('--fallback-file', type=Path, help='verify non-target file uses stock backend in the same process')
    p.add_argument('--source-cache', type=Path)
    p.add_argument('--output', type=Path)
    p.add_argument('--ramp', action='store_true', help='assert known full-range 256x64 ramp fixture')
    p.add_argument('--bars', action='store_true', help='assert known 192x96 limited-range 16/128/235 fixture pixels')
    p.add_argument('--check', action='store_true', help='bounded pixel/control probe instead of interactive Pivra')
    a = p.parse_args()
    if (a.bars or a.ramp) and not a.check:
        p.error('--bars/--ramp require --check')
    video = a.video.resolve(strict=True)
    stream = json.loads(subprocess.run(['ffprobe', '-v', 'error', '-show_streams',
        '-of', 'json', str(video)], capture_output=True, text=True, check=True, timeout=20).stdout)['streams']
    videos = [s for s in stream if s.get('codec_type') == 'video']
    if len(videos) != 1 or not eligible(videos[0]):
        p.error('prototype restricted to one explicit 8-bit BT709 SDR video stream; use stock Pivra for this source')
    root = a.output or (Path(tempfile.mkdtemp(prefix='pivra-owned-gpu-')) / 'session')
    module = build(root, a.source_cache)
    (root / 'metadata.json').write_text(json.dumps(videos[0], indent=2) + '\n')
    env = os.environ.copy()
    for key in ('GTK_MEDIA', 'GTK_PATH', 'GIO_EXTRA_MODULES', 'PIVRA_VIDEO_TEST_MODULE_DIR',
                'PIVRA_GTK_BT1886_CPU', 'PIVRA_GTK_COLOR_CPU_TRANSPORT', 'LD_PRELOAD'):
        env.pop(key, None)
    version = subprocess.run(['pkg-config', '--modversion', 'gtk4'], capture_output=True,
                             text=True, check=True).stdout.strip()
    env['GSK_RENDERER'] = 'ngl' if int(version.split('.')[1]) < 20 else 'gl'
    env.update(PIVRA_GPU_VIDEO_TARGET=str(video), PIVRA_GPU_VIDEO_MODULE_DIR=str(module.parent),
               GST_DEBUG_NO_COLOR='1')
    if a.check:
        subprocess.run(['cargo', 'build', '--locked', '--example', 'video_color_probe'], cwd=ROOT, check=True)
        # The production factory constructs the subtype explicitly. No global
        # GTK_MEDIA override. Pixel readback is test-only, not playback.
        argv = [str(ROOT / 'target/debug/examples/video_color_probe'), str(video), '--check-controls']
        if a.ramp:
            argv.append('--check-bt1886-ramp')
        if a.fallback_file:
            argv.extend(['--verify-stock-fallback', str(a.fallback_file.resolve(strict=True))])
    else:
        subprocess.run(['cargo', 'build', '--locked'], cwd=ROOT, check=True)
        argv = [str(ROOT / 'target/debug/pivra'), str(video)]
    linkage = subprocess.run(['ldd', argv[0]], env=env, capture_output=True, text=True, check=True).stdout
    paths = [line.split('=>', 1)[1].split()[0] for line in linkage.splitlines()
             if 'libgtk-4.so' in line and '=>' in line]
    if len(paths) != 1:
        raise RuntimeError('cannot establish stock GTK library provenance')
    data = Path(paths[0]).read_bytes()
    if b'video-bt1886 GPU CICP op' in data:
        raise RuntimeError('refusing patched GTK: this probe must test stock GTK')
    (root / 'gtk-provenance.json').write_text(json.dumps(dict(path=paths[0],
        sha256=hashlib.sha256(data).hexdigest(), version=version), indent=2) + '\n')
    print('Retained session:', root, flush=True)
    print('GPU prototype applies only to this target; other files use stock GTK. Close window to finish.', flush=True)
    with (root / 'playback.log').open('w') as log:
        result = subprocess.run(argv, env=env, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT,
                                timeout=45 if a.check else None, check=False)
    text = (root / 'playback.log').read_text(errors='replace')
    print(text)
    if result.returncode:
        return result.returncode
    if 'GStreamer-CRITICAL' in text:
        raise RuntimeError('GStreamer critical during playback')
    if a.bars:
        values = re.findall(r'sample x=\d+ y=\d+ sRGB=\[(\d+), (\d+), (\d+), 255\]', text)
        assert len(values) == 3, 'missing fixture samples'
        for pixel, expected in zip(values, (0, 124, 255)):
            assert all(abs(int(c) - expected) <= 2 for c in pixel), (pixel, expected)
        print('PASS real decoded limited-range bars vs independent BT1886 reference (2 LSB tolerance)')
    if a.check and ('PIVRA_GPU_PRESENT' not in text or 'PIVRA_GPU_STOP' not in text):
        raise RuntimeError('missing synchronized texture presentation or filter shutdown')
    if 'PIVRA_GPU_DRAW' not in text:
        raise RuntimeError('no actual GPU filter draw was observed')
    print('GPU API path executed; native hardware/visual, Vulkan interop and audio acceptance remain required.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
