"""Bounded EXIF extraction; all JPEG pixel decoding is a Bwrap child."""
import argparse
import hashlib
from pathlib import Path
import signal
import subprocess
import tempfile
import time

LIMIT = 64 * 1024 * 1024

def thumbnail(data):
    if len(data) > LIMIT or not data.startswith(b'\xff\xd8'):
        raise ValueError('not bounded JPEG')
    offset = 2
    found = None
    while offset < len(data):
        if data[offset] != 255:
            raise ValueError('invalid JPEG marker')
        while offset < len(data) and data[offset] == 255:
            offset += 1
        if offset >= len(data):
            raise ValueError('truncated marker')
        marker = data[offset]
        offset += 1
        if marker in (0xD9, 0xDA):
            return found
        if marker == 0xD8 or marker == 0 or 0xD0 <= marker <= 0xD7:
            raise ValueError('unexpected marker')
        if offset + 2 > len(data):
            raise ValueError('truncated segment')
        size = int.from_bytes(data[offset:offset+2], 'big')
        if size < 2 or offset + size > len(data):
            raise ValueError('segment bounds')
        payload = data[offset+2:offset+size]
        if marker == 0xE1 and payload.startswith(b'Exif\0\0'):
            if found is not None:
                raise ValueError('ambiguous EXIF thumbnails')
            found = exif_thumbnail(payload[6:])
        offset += size
    raise ValueError('missing scan/end marker')

def exif_thumbnail(tiff):
    if len(tiff) < 8 or tiff[:2] not in (b'II', b'MM'):
        raise ValueError('TIFF header')
    order = '<' if tiff[:2] == b'II' else '>'
    def integer(pos, length):
        if pos < 0 or pos + length > len(tiff):
            raise ValueError('EXIF bounds')
        return int.from_bytes(tiff[pos:pos+length], 'little' if order == '<' else 'big')
    if integer(2,2) != 42:
        raise ValueError('TIFF magic')
    def directory(pos):
        if pos < 8:
            raise ValueError('IFD header overlap')
        count = integer(pos,2)
        end = pos+2+count*12
        if count > 128 or end+4 > len(tiff):
            raise ValueError('IFD bounds/count')
        tags = {}
        for at in range(pos+2,end,12):
            tag, kind = integer(at,2), integer(at+2,2)
            if tag in tags:
                raise ValueError('duplicate EXIF tag')
            tags[tag] = (kind,integer(at+4,4),integer(at+8,4),at+8)
        return tags, integer(end,4), end+4
    first = integer(4,4)
    _, second, first_end = directory(first)
    if not second:
        return None
    if second < first_end:
        raise ValueError('cyclic/overlapping IFD')
    tags, following, table_end = directory(second)
    if following:
        raise ValueError('unexpected further IFD')
    if 0x201 not in tags and 0x202 not in tags:
        return None
    def scalar(tag, kind):
        value = tags.get(tag)
        if value is None or value[:2] != (kind,1):
            raise ValueError('thumbnail field type/count')
        return integer(value[3],2) if kind == 3 else value[2]
    if scalar(0x103,3) != 6:
        raise ValueError('thumbnail is not JPEG')
    start, length = scalar(0x201,4), scalar(0x202,4)
    if start < table_end or length < 4 or start+length > len(tiff):
        raise ValueError('thumbnail offset/length')
    result = tiff[start:start+length]
    if not result.startswith(b'\xff\xd8') or not result.endswith(b'\xff\xd9'):
        raise ValueError('thumbnail JPEG envelope')
    return result

def decode(binary, data, mode):
    command = ['bwrap','--unshare-all','--die-with-parent','--new-session',
               '--cap-drop','ALL','--clearenv','--ro-bind','/usr','/usr',
               '--symlink','usr/lib','/lib','--symlink','usr/lib64','/lib64',
               '--ro-bind-try','/etc/ld.so.cache','/etc/ld.so.cache',
               '--proc','/proc','--dev','/dev','--tmpfs','/tmp',
               '--ro-bind',str(binary),'/decoder','/decoder',str(mode)]
    start = time.monotonic_ns()
    import resource
    def limits():
        resource.setrlimit(resource.RLIMIT_FSIZE, (100*1024*1024,100*1024*1024))
        resource.setrlimit(resource.RLIMIT_AS, (512*1024*1024,512*1024*1024))
        resource.setrlimit(resource.RLIMIT_CPU, (10,10))
    out = tempfile.TemporaryFile()
    err = tempfile.TemporaryFile()
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=out,
                             stderr=err, start_new_session=True, preexec_fn=limits)
    try:
        child.communicate(data, timeout=20)
    except (subprocess.TimeoutExpired, KeyboardInterrupt):
        # Cancellation kills the dedicated sandbox group, never a Pivra loader.
        import os
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.communicate(timeout=2)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.communicate()
        out.close()
        err.close()
        raise
    out.seek(0)
    output = out.read(96_000_001 + 1024)
    err.seek(0)
    errors = err.read(4096)
    out.close()
    err.close()
    if len(output) > 96_000_000 + 1024:
        raise ValueError('decoder output exceeds bound')
    elapsed = (time.monotonic_ns()-start)/1e6
    if child.returncode:
        print(errors.decode(errors='replace').strip())
        if b'bwrap:' in errors:
            raise SystemExit(77)
        return None
    lines = output.split(b'\n',3)
    if len(lines) != 4 or lines[0] != b'P6' or lines[2] != b'255':
        raise ValueError('decoder output header')
    width, height = map(int,lines[1].split())
    if not 0 < width <= 32768 or not 0 < height <= 32768 or len(lines[3]) != width*height*3:
        raise ValueError('decoder output bounds')
    print(errors.decode().strip(), f'sandbox_complete_ms={elapsed:.3f}')
    return output, (width,height), elapsed

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--decoder',type=Path,required=True)
    parser.add_argument('jpeg',type=Path)
    args = parser.parse_args()
    with args.jpeg.open('rb') as source:
        data = source.read(LIMIT+1)
    digest = hashlib.sha256(data).hexdigest()
    embedded = thumbnail(data)
    print(f'SOURCE {args.jpeg} sha256={digest}; pixel decode only in Bwrap')
    with tempfile.TemporaryDirectory(prefix='pivra-jpeg-preview-') as folder:
        # These are bounded generated RGB artifacts, not decoded on the host.
        measurements = []
        for method, payload, mode in [('embedded',embedded,0),('reduced',data,8)]:
            if payload is None:
                print(method,'NO PREVIEW: controlled fallback to reduced/full'); continue
            result = decode(args.decoder.resolve(),payload,mode)
            if result is None:
                print(method,'UNSUPPORTED/FAIL'); continue
            output, size, elapsed = result
            artifact = Path(folder)/f'{method}.ppm'
            artifact.write_bytes(output)
            print(method, 'FRAME READY', size, 'rgb_bytes=',size[0]*size[1]*3,
                  'before full baseline starts; onscreen presentation NOT MEASURED')
            measurements.append((method,size,elapsed))
        # A single separate full baseline, only after both preview attempts.
        if measurements:
            baseline = decode(args.decoder.resolve(),data,1)
            if baseline is not None:
                _, full_size, full_ms = baseline
                for method, size, elapsed in measurements:
                    smaller = size[0]*size[1] < full_size[0]*full_size[1]
                    print(method, 'smaller=',smaller,'full_ms=',full_ms,'preview_ms=',elapsed,
                          'single sample is not a speedup claim; orientation not applied')
    print('Observation complete; no production integration or automatic acceptance')

if __name__ == '__main__':
    main()
