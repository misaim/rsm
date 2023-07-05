all:
	echo "No Target. Try make run"

build: src/main.rs style.toml
	cargo build 

release: clean
	mkdir -p bin
	rm -rf bin/*
	cargo build --release
	cp target/release/rsm bin/rsm

run: build
	./target/debug/rsm

clean:
	cargo clean
	rm -rf bin/

