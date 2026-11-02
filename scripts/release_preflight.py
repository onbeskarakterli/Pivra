#!/usr/bin/env python3
"""Release preflight for Pivra; stdlib-only and safe by default."""
import argparse
from pathlib import Path
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
ARCH = ROOT / "packaging" / "arch"

def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout.strip()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="Release version, e.g. 0.2.5")
    parser.add_argument("--clean-old-packages", action="store_true", help="Delete ONLY old, untracked package archives under packaging/arch")
    args = parser.parse_args()
    version = args.version
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        parser.error("Expected a numeric X.Y.Z version")

    errors = []
    try:
        branch = git("branch", "--show-current")
        if branch != "dev":
            errors.append(f"Current branch is {branch!r}, expected 'dev'")
        tracked_status = git("status", "--porcelain", "--untracked-files=no")
        if tracked_status:
            errors.append("Tracked working tree is dirty; commit or review changes first")
    except (subprocess.CalledProcessError, FileNotFoundError) as exc:
        errors.append(f"Git status unavailable: {exc}")

    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text())
    if cargo["package"]["version"] != version:
        errors.append("Cargo.toml version differs from release version")
    lock = (ROOT / "Cargo.lock").read_text()
    if not re.search(r'(?m)^name = "pivra"\nversion = "' + re.escape(version) + r'"$', lock):
        errors.append("Cargo.lock pivra package version differs")

    recipe = (ARCH / "PKGBUILD").read_text()
    if not re.search(r"(?m)^pkgver=" + re.escape(version) + r"$", recipe):
        errors.append("PKGBUILD pkgver differs")
    if not re.search(r"(?m)^options=\(['\"]!debug['\"]\)$", recipe):
        errors.append("PKGBUILD must contain options=('!debug')")
    if not re.search(r"(?m)^pkgrel=1$", recipe):
        errors.append("Check PKGBUILD pkgrel (expected 1 for a new upstream release)")

    readme = (ROOT / "README.md").read_text()
    if f"v{version}" not in readme and f"version {version}" not in readme:
        errors.append("README does not mention target release")
    notes = ROOT / "docs" / "releases" / f"PIVRA_V{version}_RELEASE_NOTES.md"
    if not notes.is_file():
        errors.append(f"Missing release notes: {notes.relative_to(ROOT)}")
    else:
        s = notes.read_text().lower()
        if "unreleased" in s or "development candidate" in s:
            errors.append("Release notes still contain draft/unreleased wording")
    if "unreleased" in readme.lower() and f"v{version}, unreleased" in readme.lower():
        errors.append("README still labels the release as unreleased")

    old = sorted(set(ARCH.glob("pivra-*.pkg.tar.*")) | set(ARCH.glob("pivra-*.tar.gz")))
    old = [p for p in old if not p.name.startswith(f"pivra-{version}-") and p.name != f"pivra-{version}.tar.gz"]
    for p in old:
        if p.is_symlink() or not p.is_file():
            errors.append(f"Unsafe old archive entry: {p.name}")
            continue
        # Delete only files Git has not tracked. No recursive deletion.
        try:
            tracked = subprocess.run(["git", "-C", str(ROOT), "ls-files", "--error-unmatch", "--", str(p.relative_to(ROOT))], capture_output=True).returncode == 0
        except OSError:
            tracked = True
        if tracked:
            errors.append(f"Refusing to remove tracked archive: {p.name}")
        elif args.clean_old_packages:
            p.unlink()
            print(f"CLEANED: {p.relative_to(ROOT)}")
        else:
            print(f"OLD PACKAGE (not deleted): {p.relative_to(ROOT)}")

    print("PRESERVED: .pivra-cache/, release-source.conf, .SRCINFO, current-version archives, and build directories")
    if errors:
        for msg in errors:
            print(f"FAIL: {msg}", file=sys.stderr)
        return 1
    print(f"PASS: Pivra v{version} source/document/package preflight")
    print("NEXT: native fmt, Clippy, tests, verified commit archive, makepkg, and explicit publication approval")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
