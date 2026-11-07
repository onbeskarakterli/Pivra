#!/usr/bin/env python3
"""Real production discoverer routing with ffprobe-verified synthetic metadata.

PQ/HLG files only test tag-based exclusion; they do not validate HDR pixels.
Requires an existing GTK display and ffmpeg/ffprobe, not a custom GTK build.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]

def run(*args):
    return subprocess.run(args, check=True, capture_output=True, text=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/debug/examples/integrated_video_probe')
    parser.add_argument('--output', type=Path, required=True)
    a = parser.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    spec = importlib.util.spec_from_file_location('video_reference', ROOT / 'tests/video_color/backend_probe.py')
    reference = importlib.util.module_from_spec(spec); spec.loader.exec_module(reference)
    reference.fixtures(a.output)
    for name, transfer, code in [('pq', 'smpte2084', 16), ('hlg', 'arib-std-b67', 18)]:
        path = a.output / (name + '.mp4')
        run('ffmpeg', '-hide_banner', '-loglevel', 'error', '-i', str(a.output / 'bt709.mp4'),
            '-c:v', 'copy', '-bsf:v', f'h264_metadata=colour_primaries=9:transfer_characteristics={code}:matrix_coefficients=9',
            '-color_primaries', 'bt2020', '-color_trc', transfer, '-colorspace', 'bt2020nc', str(path))
        info = json.loads(run('ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(path)).stdout)['streams'][0]
        assert info['color_transfer'] == transfer and info['color_primaries'] == 'bt2020', info
        print('VERIFIED synthetic exclusion metadata', name, info['color_transfer'], info['color_primaries'], flush=True)
    run('ffmpeg', '-hide_banner', '-loglevel', 'error', '-i', str(a.output / 'bt709.mp4'),
        '-c:v', 'libvpx-vp9', str(a.output / 'vp9.webm'))
    subtitle = a.output / 'subtitle.srt'
    subtitle.write_text('1\n00:00:00,000 --> 00:00:01,000\nRouting fixture\n')
    run('ffmpeg', '-hide_banner', '-loglevel', 'error', '-i', str(a.output / 'bt709.mp4'), '-i', str(subtitle),
        '-c:v', 'copy', '-c:s', 'srt', str(a.output / 'subtitles.mkv'))
    info = json.loads(run('ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(a.output / 'subtitles.mkv')).stdout)
    assert any(s['codec_type'] == 'subtitle' for s in info['streams']), info
    routes = [('gpu', name) for name in ('bt709.mp4', 'full709.mp4')]
    routes += [('stock', name) for name in ('srgb.mp4', 'unknown.mp4', 'pq.mp4', 'hlg.mp4', 'vp9.webm', 'subtitles.mkv')]
    args = [str(a.binary.resolve()), '--routing-only', *[kind+'='+str((a.output/name).resolve()) for kind,name in routes],
            'stock='+str(ROOT / 'tests/fixtures/images/wide.png')]
    return subprocess.run(args, check=False, timeout=40).returncode

if __name__ == '__main__':
    raise SystemExit(main())
