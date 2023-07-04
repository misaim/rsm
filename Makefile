all:
	echo "Building without server, -Os"

build: src/main.rs style.toml
	cargo build 

release: clean build
	mkdir bin
	cp target/debug/rsm bin/rsm

run: build
	./target/debug/rsm

clean:
	cargo clean
