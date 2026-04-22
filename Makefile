BINARY_NAME := nuwax-upload

.PHONY: build release install uninstall clean

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .

uninstall:
	cargo uninstall $(BINARY_NAME)

clean:
	cargo clean
