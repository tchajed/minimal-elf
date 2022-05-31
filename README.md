# Creating a minimal ELF-64 file

[![Cargo Build & Test](https://github.com/tchajed/minimal-elf/actions/workflows/build.yml/badge.svg)](https://github.com/tchajed/minimal-elf/actions/workflows/build.yml)

I wanted to create a minimal binary, that doesn't issue any syscalls. That
turned into creating a minimal binary, which led to this [blog
post](https://www.muppetlabs.com/~breadbox/software/tiny/teensy.html).
Unfortunately the post creates a 32-bit ELF file, and it would be nicer to use
64-bit. I also wanted to write it in Rust, to learn some Rust.

There are three stages of minimization here: `tiny.c`, `tiny.s`, and then a Rust
program that generates a binary directly.
