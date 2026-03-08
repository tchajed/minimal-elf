ALL_TARGETS = tiny_c tiny_asm tiny_asm_opt tiny

CC ?= clang
STRIP ?= strip
OBJCOPY ?= objcopy

all: $(ALL_TARGETS)

tiny_c: tiny.c
	$(CC) -O2 -static $< -o $@

tiny_asm: tiny.s
	$(CC) -nostartfiles -nostdlib -static $< -o $@
	$(STRIP) $@

tiny_asm_opt: tiny.s
	$(CC) -s -Wl,--build-id=none -z noseparate-code -nostartfiles -nostdlib -static $< -o $@
	$(OBJCOPY) --strip-section-headers $@ $@

tiny: src/main.rs
	cargo run

clean:
	rm -f $(ALL_TARGETS)
