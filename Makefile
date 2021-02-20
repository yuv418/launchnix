all:
	mkdir -p target
	cp -r nix target/debug
	cargo run

clean:
	cargo clean

