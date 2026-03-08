ALL_TARGETS = tiny_c tiny_asm tiny_asm_opt tiny

all: $(ALL_TARGETS)

tiny_c: tiny.c
	clang -O2 -static $< -o $@

tiny_asm: tiny.s
	clang -nostartfiles -nostdlib -static $< -o $@
	strip $@

tiny_asm_opt: tiny.s
	clang -s -Wl,--build-id=none -z noseparate-code -nostartfiles -nostdlib -static $< -o $@
	objcopy --strip-section-headers $@ $@

tiny: src/main.rs
	cargo run

clean:
	rm -f $(ALL_TARGETS)
