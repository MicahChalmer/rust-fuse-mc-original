.PHONY: install test

install: bin/hello_fs

CLEAN := rm -rf ./.rust ./build ./lib ./bin

test: bin/test
	env RUST_LOG=test,rust_fuse ./bin/test

bin/hello_fs: $(shell git ls-files src/rust_fuse src/examples '*.rs')
	$(CLEAN); rustpkg install examples/hello_fs

bin/test: bin/hello_fs $(shell git ls-files src/test '*.rs')
	rustc --test --out-dir bin src/test/test.rs
