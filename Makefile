.PHONY: install test

install: bin/hello_fs

test: bin/test
	env RUST_LOG=test=4,rust_fuse=4 ./bin/test

bin/hello_fs: $(shell git ls-files src/rust_fuse src/examples '*.rs')
	rm .rust/*.json; rustpkg install examples/hello_fs

bin/test: bin/hello_fs $(shell git ls-files src/test '*.rs')
	rustc --test --out-dir bin src/test/test.rs
