# Common Linux source build frontend for Pivra.
# Dependencies are installed using the distribution's own package manager.
SHELL := /bin/bash
.DEFAULT_GOAL := help
PREFIX ?= /usr/local
DESTDIR ?=

.PHONY: help deps check build test stage install release-preflight

help:
	@printf '%s\n' 'Pivra Linux build targets:' \
	  '  make deps        Install native build/runtime dependencies (sudo required)' \
	  '  make check       Check tools and native development libraries' \
	  '  make build       Build Pivra and the GNOME thumbnail helper' \
	  '  make test        Run Rust tests' \
	  '  make stage       Stage files under DESTDIR (requires DESTDIR)' \
	  '  make install     Install already-built files (PREFIX=/usr/local by default)' \
\t  '  make release-preflight VERSION=X.Y.Z  Audit a release without deleting files'

deps:
	@bash packaging/linux/dependencies.sh install

check:
	@bash packaging/linux/dependencies.sh check

build: check
	@bash packaging/linux/build-install.sh build

test: check
	@PIVRA_DISTRIBUTION_BUILD=1 cargo test --locked

stage:
	@test -n "$(DESTDIR)" || { echo 'ERROR: Set DESTDIR=/absolute/staging/path for make stage' >&2; exit 2; }
	@PREFIX="$(PREFIX)" DESTDIR="$(DESTDIR)" bash packaging/linux/build-install.sh install

install:
	@PREFIX="$(PREFIX)" DESTDIR="$(DESTDIR)" bash packaging/linux/build-install.sh install

# RELEASE_CLEAN_OLD=1 removes only untracked, old-version package archives.
# Cache/config/build directories are deliberately preserved.
release-preflight:
	@test -n "$(VERSION)" || { echo 'ERROR: Set VERSION=X.Y.Z' >&2; exit 2; }
	@python3 scripts/release_preflight.py --version "$(VERSION)" $(if $(filter 1,$(RELEASE_CLEAN_OLD)),--clean-old-packages,)
