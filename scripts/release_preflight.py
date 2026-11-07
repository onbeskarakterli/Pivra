#!/usr/bin/env python3
"""Release preflight for Pivra; stdlib-only and safe by default."""
import argparse
import importlib.util
from pathlib import Path
import re
import subprocess
import sys
import tomllib

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
ARCH = ROOT / "packaging" / "arch"

def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True).stdout.strip()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="Release version, e.g. 0.2.5")
    parser.add_argument("--binary", type=Path, help="Inspect an actual GPU-enabled distribution ELF")
    parser.add_argument("--staging-root", type=Path, help="Inspect an actual PREFIX=/usr DESTDIR staging tree")
    parser.add_argument("--candidate", action="store_true", help="Validate the exact target candidate; publication/source approval remain separate")
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
    source_version = cargo["package"]["version"]
    checked_version = version
    if source_version != checked_version:
        errors.append("Cargo.toml version differs from release version")
    lock = (ROOT / "Cargo.lock").read_text()
    if not re.search(r'(?m)^name = "pivra"\nversion = "' + re.escape(checked_version) + r'"$', lock):
        errors.append("Cargo.lock pivra package version differs")

    recipe = (ARCH / "PKGBUILD").read_text()
    if not re.search(r"(?m)^pkgver=" + re.escape(checked_version) + r"$", recipe):
        errors.append("PKGBUILD pkgver differs")
    if not re.search(r"(?m)^options=\(['\"]!debug['\"]\)$", recipe):
        errors.append("PKGBUILD must contain options=('!debug')")
    if not re.search(r"(?m)^pkgrel=1$", recipe):
        errors.append("Check PKGBUILD pkgrel (expected 1 for a new upstream release)")

    readme = (ROOT / "README.md").read_text()
    candidate_marker = f"**Upcoming release candidate: v{version}.**"
    stable_marker = f"**Current stable release: v{version}.**"
    if args.candidate:
        if candidate_marker not in readme and stable_marker not in readme:
            errors.append("README must identify the exact target candidate or current stable release")
        report = ROOT / "docs" / "releases" / f"PIVRA_V{version}_RC_VALIDATION.md"
        if not report.is_file():
            errors.append("Missing candidate validation report")
    elif stable_marker not in readme or candidate_marker in readme:
        errors.append("Publication handoff: README must identify target as current stable, not upcoming/unreleased")
    notes_name = f"PIVRA_V{version}_RELEASE_NOTES.md"
    notes = ROOT / "docs" / "releases" / notes_name
    if not notes.is_file():
        errors.append(f"Missing release notes: {notes.relative_to(ROOT)}")
    else:
        text = notes.read_text()
        if f"# Pivra v{version}" not in text:
            errors.append("Release notes version differs from target")
        if not args.candidate and (
            "**Status: Published stable release.**" not in text
            or "release candidate" in text.splitlines()[0].lower()
        ):
            errors.append("Publication handoff: release notes must carry published stable status")
    if notes_name not in recipe:
        errors.append("PKGBUILD must install target-version release notes")

    binary = args.binary
    if args.staging_root:
        shared = args.staging_root / "usr/share"
        required = [
            f"doc/pivra/{notes_name}",
            "licenses/pivra/LICENSE", "licenses/pivra/video-adapter/COPYING.LGPL-2.1",
            "licenses/pivra/video-adapter/COPYING.LGPL-2",
            "licenses/pivra/third-party/inventory.json",
            "doc/pivra/corresponding-source/VIDEO_ADAPTER_REBUILD.md",
            "doc/pivra/corresponding-source/video-adapter/gtkgstmediafile.c",
        ]
        for name in required:
            if not (shared / name).is_file() or (shared / name).stat().st_size == 0:
                errors.append("Missing staged source/notice: " + name)
        source_dir = shared / "doc/pivra/corresponding-source"
        if not any((source_dir / name).is_file() and (source_dir / name).stat().st_size
                   for name in ("Pivra-source.tar.gz", f"pivra-{source_version}.tar.gz")):
            errors.append("Missing staged application corresponding source archive")
        if not (args.staging_root / "usr/bin/pivra-thumbnail-factory").is_file():
            errors.append("Missing staged GNOME thumbnail helper")
        binary = binary or args.staging_root / "usr/bin/pivra"
    if binary:
        spec = importlib.util.spec_from_file_location("video_elf_contract", ROOT / "packaging/linux/verify-video-binary.py")
        verifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(verifier)
        try:
            verifier.verify(binary)
            output = subprocess.run([str(binary.resolve()), "--version"], capture_output=True, text=True, timeout=10, check=True)
            if output.stdout.strip() != f"Pivra {version}":
                raise ValueError(f"staged binary version differs: {output.stdout.strip()!r}")
            print("PASS: actual GPU-enabled distribution ELF and exact binary version")
        except (ValueError, OSError, subprocess.SubprocessError) as exc:
            errors.append("GPU ELF validation: " + str(exc))
    elif not args.candidate:
        errors.append("Final release preflight requires --binary or --staging-root")

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
    if args.candidate:
        print(f"PASS: v{version} preparation preflight; actual source/package version {source_version}")
        print("BLOCKED publication: immutable source/hash approval, actual makepkg/package acceptance and stable-documentation handoff remain required")
        return 0
    print(f"PASS: Pivra v{version} source/document/package preflight")
    print("NEXT: native fmt, Clippy, tests, verified commit archive, makepkg, and explicit publication approval")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
