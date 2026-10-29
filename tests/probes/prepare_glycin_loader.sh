#!/usr/bin/env bash
# Build exactly the official JPEG/PNG-capable image-rs loader; no system install.
set -euo pipefail
repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
test "$(git -C "$repo" branch --show-current)" = dev
root=${PIVRA_IMAGE_ENV:-/workspace/pivra-environment/piv09}
activation=${PIVRA_ENV_ACTIVATE:-/workspace/pivra-environment/gtk-activate.sh}
[[ "$root" = /* ]] || { echo 'PIVRA_IMAGE_ENV must be absolute' >&2; exit 2; }
source "$activation"
pkg-config --exists libseccomp
source_dir="$root/glycin-source"
commit=724f18c5f3519885995428a7e2e6e5df9ca2ab27
mkdir -p "$root"
if [[ ! -d "$source_dir" ]]; then
    git clone --depth 1 --branch 2.0.0 https://github.com/GNOME/glycin.git "$source_dir"
fi
test "$(git -C "$source_dir" rev-parse HEAD)" = "$commit"
test -z "$(git -C "$source_dir" status --porcelain)"
cargo +stable build --locked --manifest-path "$source_dir/Cargo.toml" \
    --target-dir "$root/loader-target" -p glycin-image-rs -j 2
prefix="$root/loader-prefix"
mkdir -p "$prefix/libexec" "$prefix/share/glycin-loaders/2+/conf.d"
install -m 0755 "$root/loader-target/debug/glycin-image-rs" "$prefix/libexec/glycin-image-rs"
work=$(mktemp -d "$root/loader-config-XXXXXXXX")
trap 'rm -rf "$work"' EXIT
cat > "$work/pivra-jpeg-png.conf" <<CONF
[loader:image/jpeg]
Exec=$prefix/libexec/glycin-image-rs

[loader:image/png]
Exec=$prefix/libexec/glycin-image-rs
CONF
printf 'source %q\nexport GLYCIN_DATA_DIR=%q\n' "$activation" "$prefix/share" > "$work/image-activate.sh"
install_config() {
    if [[ -e "$2" ]] && ! cmp -s "$1" "$2"; then
        echo "Refusing to overwrite modified config: $2" >&2
        exit 1
    fi
    install -m 0644 "$1" "$2"
}
install_config "$work/pivra-jpeg-png.conf" "$prefix/share/glycin-loaders/2+/conf.d/pivra-jpeg-png.conf"
install_config "$work/image-activate.sh" "$root/image-activate.sh"
python3 - "$root" "$commit" <<'PY'
import hashlib,json,sys,tomllib
from pathlib import Path
root=Path(sys.argv[1]); source=root/'glycin-source'
version=tomllib.loads((source/'Cargo.toml').read_text())['workspace']['package']['version']
assert version=='2.0.0'
binary=root/'loader-prefix/libexec/glycin-image-rs'
manifest={'source':'https://github.com/GNOME/glycin.git','tag':'2.0.0','commit':sys.argv[2],
          'loader':'glycin-image-rs','version':version,'config_protocol_family':'2+',
          'configured_loader_mimes':['image/jpeg','image/png'],'configured_editors':[],
          'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
          'upstream_lock_sha256':hashlib.sha256((source/'Cargo.lock').read_bytes()).hexdigest(),
          'scope':'Unmodified upstream image-rs bundle only; discovery, sandbox and pixels need separate probes'}
(root/'loader-build.json').write_text(json.dumps(manifest,indent=2)+'\n')
PY
echo "Prepared image-rs loader and JPEG/PNG-only config under $prefix; NOT a decode PASS"
