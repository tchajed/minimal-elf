all: tiny_c tiny_asm tiny

tiny_c: tiny.c
	clang -O2 -static $< -o $@

tiny_asm: tiny.s
	clang -nostartfiles -nostdlib -static $< -o $@

tiny: src/main.rs
	cargo run
