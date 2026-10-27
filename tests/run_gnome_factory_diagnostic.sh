#!/usr/bin/env bash
# Same native GNOME adapter and resource limits, isolated cache, no installation.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
if [[ $# != 1 ]]; then
  echo "usage: bash tests/run_gnome_factory_diagnostic.sh /path/to/photo.jpg" >&2
  exit 2
fi
if ! pkg-config --exists gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0; then
  echo "NOT RUN: GNOME desktop-4 development API unavailable" >&2
  exit 77
fi
scratch=$(mktemp -d -t pivra-gnome-factory-diagnostic-XXXXXXXX)
trap 'rm -rf -- "$scratch"' EXIT
read -r -a cflags <<< "$(pkg-config --cflags gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0)"
read -r -a libs <<< "$(pkg-config --libs gnome-desktop-4 gio-2.0 gobject-2.0 gdk-pixbuf-2.0)"
cc -O2 -Wall -Wextra -Werror "${cflags[@]}" native/thumbnail_factory.c -o "$scratch/helper" "${libs[@]}"
python3 - "$scratch" "$1" <<'PY'
import configparser
import os
from pathlib import Path
import signal
import subprocess
import sys

scratch = Path(sys.argv[1])
source = Path(sys.argv[2]).absolute()
info = source.stat()
if not source.is_file():
    raise SystemExit('FAIL: source must be a regular file')
# Use the same GIO content-type query as production, not a suffix-based guess.
query = subprocess.run(['gio', 'info', '-a', 'standard::content-type', str(source)],
                       env={**os.environ, 'LC_ALL': 'C'}, check=True,
                       capture_output=True, text=True, timeout=5)
mime = next((line.split('standard::content-type:', 1)[1].strip()
             for line in query.stdout.splitlines() if 'standard::content-type:' in line), None)
if not mime or '/' not in mime:
    raise SystemExit('FAIL: GIO MIME query did not return a MIME type')
print(f'INPUT uri={source.as_uri()!r} mime={mime!r} bytes={info.st_size} mtime={int(info.st_mtime)}', flush=True)
# List candidates only; actual selection is certified by the GNOME debug command.
data_dirs = [os.environ.get('XDG_DATA_HOME', str(Path.home()/'.local/share'))]
data_dirs += os.environ.get('XDG_DATA_DIRS', '/usr/local/share:/usr/share').split(':')
for directory in data_dirs:
    folder = Path(directory)/'thumbnailers'
    for path in sorted(folder.glob('*.thumbnailer'))[:128]:
        try:
            if path.stat().st_size > 65536:
                continue
            parser = configparser.ConfigParser(interpolation=None)
            parser.read_string(path.read_text())
            entry = parser['Thumbnailer Entry']
            if mime in entry.get('MimeType', '').split(';'):
                print(f'CANDIDATE path={str(path)!r} TryExec={entry.get("TryExec")!r} Exec={entry.get("Exec")!r}', flush=True)
        except (OSError, UnicodeError, configparser.Error, KeyError) as error:
            print(f'CANDIDATE_READ_ERROR path={str(path)!r} error={str(error)!r}', flush=True)
# Preserve session/data/config selection. Only cache/temp are private; no shared
# thumbnail, failed marker, or source file is removed or replaced by this probe.
cache = scratch/'cache'
cache.mkdir(mode=0o700)
env = {**os.environ, 'XDG_CACHE_HOME': str(cache), 'TMPDIR': str(scratch),
       'PIVRA_TRACE_GNOME_THUMB': '1'}
process = subprocess.Popen([str(scratch/'helper'), source.as_uri(), mime,
                            str(int(info.st_mtime)), 'missing'], env=env,
                           start_new_session=True)
try:
    status = process.wait(timeout=20)
except (subprocess.TimeoutExpired, KeyboardInterrupt):
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait()
    print('FAIL: factory timeout/interruption; process group killed and reaped', flush=True)
    raise SystemExit(1)
print(f'FACTORY_EXIT {status}; ' + ('PASS real GNOME factory generation' if status == 0
                                else 'FAIL GNOME factory (see command/child diagnostics above)'), flush=True)
raise SystemExit(0 if status == 0 else 1)
PY
