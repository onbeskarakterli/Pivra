#!/usr/bin/env python3
"""Generate private H264 limited bars and a full-range ramp for GPU assertions."""
import argparse
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    for name, width, height, full in [('bars', 192, 96, False), ('ramp', 256, 64, True)]:
        row = bytes(range(256)) if full else bytes([16] * 64 + [128] * 64 + [235] * 64)
        source = args.output / (name + '.yuv')
        source.write_bytes(row * height + bytes([128]) * (width * height // 2))
        subprocess.run(['ffmpeg', '-hide_banner', '-loglevel', 'error', '-f', 'rawvideo',
            '-pixel_format', 'yuv420p', '-video_size', f'{width}x{height}', '-framerate', '8',
            '-i', str(source), '-vf', 'loop=loop=15:size=1:start=0', '-frames:v', '16',
            '-c:v', 'libx264', '-profile:v', 'high', '-crf', '1', '-pix_fmt', 'yuv420p',
            '-x264-params', 'colorprim=bt709:transfer=bt709:colormatrix=bt709:fullrange=' + ('on' if full else 'off'),
            str(args.output / (name + '.mp4'))], check=True)


if __name__ == '__main__':
    main()
