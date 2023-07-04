all:
	echo "Building without server, -Os"

build: src/main.rs style.toml
	cargo build 

release:
	rm -rf bin/*
	cargo build --release
	cp target/release/rsm bin/rsm

run: build
	./target/debug/rsm

clean:
	cargo clean

