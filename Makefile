# Build Meeple and publish it.
#
#   make           build the release bundle into release/
#   make deploy    build, then upload to the web server (https://meeple.jb3.dev)
#   make serve     run the local dev server
#   make test      run the native tests
#   make clean     remove build output

HOST       ?= odin.host.jb3.dev
REMOTE_DIR ?= /var/www/meeple/
# meeple.jb3.dev serves the bundle from its root.
PUBLIC_URL ?= /
# Its own output and target directories, so it doesn't fight a running
# `trunk serve` over dist/ or over wasm-opt's scratch file in target/ (which
# once shipped an empty .wasm).
OUT        := release
TARGET_DIR := target/bundle
NATIVE     := $(shell rustc -vV | sed -n 's/^host: //p')

.PHONY: all build deploy serve test clean

all: build

build:
	rustup target add wasm32-unknown-unknown
	CARGO_TARGET_DIR=$(TARGET_DIR) trunk build --release --dist $(OUT) --public-url $(PUBLIC_URL)
	@test -s $(OUT)/*_bg.wasm || { echo "error: $(OUT) has an empty .wasm" >&2; exit 1; }
	@# Beacon polled by web/update.js: "<everything> <shell>". The first changes
	@# with any file; the second ignores the wasm and its glue (and the hashes
	@# index.html quotes for them), so the page can tell a build it can swap in
	@# place from one that needs a reload.
	cd $(OUT) && { \
	  all=$$(find . -type f ! -name version.txt ! -name .trunk-lock | LC_ALL=C sort | xargs shasum -a 256 | shasum -a 256 | cut -c1-16); \
	  shell=$$( { sed -E 's/meeple-[0-9a-f]+/meeple-X/g; s/integrity="[^"]*"//g' index.html; \
	    find . -type f ! -name version.txt ! -name .trunk-lock ! -name index.html ! -name 'meeple-*.js' ! -name 'meeple-*.wasm' | LC_ALL=C sort | xargs shasum -a 256; } | shasum -a 256 | cut -c1-16); \
	  echo "$$all $$shell" > version.txt; }

# Upload the bundle, removing files left over from older builds. The remote
# side runs rsync under sudo so it can write to /var/www.
deploy: build
	rsync -avz --delete --exclude=.trunk-lock --delete-excluded --rsync-path="sudo rsync" $(OUT)/ $(HOST):$(REMOTE_DIR)

serve:
	trunk serve --release

test:
	cargo test --target $(NATIVE)

clean:
	cargo clean
	rm -rf $(OUT) dist
