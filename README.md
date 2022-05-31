# Creating a minimal ELF-64 file

[![Cargo Build & Test](https://github.com/tchajed/minimal-elf/actions/workflows/build.yml/badge.svg)](https://github.com/tchajed/minimal-elf/actions/workflows/build.yml)

I wanted to create a minimal binary, that doesn't issue any syscalls. That
turned into creating a minimal binary, which led to this [blog
post](https://www.muppetlabs.com/~breadbox/software/tiny/teensy.html).
Unfortunately the post creates a 32-bit ELF file, and it would be nicer to use
64-bit. I also wanted to write it in Rust, to learn some Rust.

There are three stages of minimization here: `tiny.c`, `tiny.s`, and then a Rust
program that generates a binary directly.

## C

```c
int main() { return 0; }
```

There's not much to do in C, but the binary it generates is a whopping 20KB:

```sh
.rwxr-xr-x 20,344 tchajed 31 May 16:25 tiny_c
```

While C is certainly a low-level language, it does have a runtime surrounding
this code. Running `objdump -d tiny_c` reveals a whole of functionality
surrounding our pithy function (which is just `xor %eax, %eax; ret`). In
particular libc provides a function `_start`, which is the actual entry point of
the file. The libc `_start` does a bunch of stuff I don't understand, including
setting up stack guards and some global initialization.

## Assembly

We can try to create a more minimal binary with the following:

tiny_bad.asm:

```asm
// tiny_bad.s
.global _start
.text
_start:
  xor %eax, %eax
  ret
```

```sh
$ clang tiny_bad.s -o tiny_bad
/usr/bin/ld: /tmp/tiny_bad-a4cf26.o: in function `_start':
(.text+0x0): multiple definition of `_start'; /usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/12.1.0/../../../../lib64/Scrt1.o:/build/glibc/src/glibc/csu/../sysdeps/x86_64/start.S:57: first defined here
/usr/bin/ld: /usr/bin/../lib64/gcc/x86_64-pc-linux-gnu/12.1.0/../../../../lib64/Scrt1.o: in function `_start':
/build/glibc/src/glibc/csu/../sysdeps/x86_64/start.S:103: undefined reference to `main'
```

Oops, the linker says there's already a `_start` and no `main`. We need to disable some
stuff provided by C; first we don't need a `_start`, we'll take care of it:

```sh
% ./tiny_bad
fish: Job 1, './tiny_bad' terminated by signal SIGSEGV (Address boundary error)
```

Oops, that doesn't look good. It turns out that one useful thing libc does for
us is to take the _return value_ from `main` (in the sense of the C calling
convention) into a call to the `exit(2)` system call to terminate the process.
Our little program calls `ret`, but there's no function to return from yet!
Instead, we should just call `exit(2)` directly. This [blog post on assembly for
Linux](https://www.cs.fsu.edu/~langley/CNT5605/2017-Summer/assembly-example/assembly.html)
was helpful, though it's nice to note that it's just walking through the Linux
source code. This is also where we depart from the blog post above, because
making a 64-bit syscall is a bit different.

```asm
// tiny.s
.globl _start
.text

_start:
	mov $60, %rax
	xorl %edi, %edi
	syscall
```

```
$ clang -nostdlib tiny.s -o tiny
$ ll tiny
.rwxr-xr-x 13,320 tchajed 31 May 16:59 tiny
$ ./tiny
```

Ok, so this works but the file is still 13KB. In fact, the binary is still doing
a bunch of other stuff:

```
$ strace ./tiny
execve("./tiny", ["./tiny"], 0x7ffd64a82ba0 /* 26 vars */) = 0
brk(NULL)                               = 0x56523aabb000
arch_prctl(0x3001 /* ARCH_??? */, 0x7fff8f6fff80) = -1 EINVAL (Invalid argument)
access("/etc/ld.so.preload", R_OK)      = -1 ENOENT (No such file or directory)
mmap(NULL, 8192, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) = 0x7f2fb1376000
arch_prctl(ARCH_SET_FS, 0x7f2fb1376a80) = 0
set_tid_address(0x7f2fb1376d50)         = 44395
set_robust_list(0x7f2fb1376d60, 24)     = 0
rseq(0x7f2fb1377420, 0x20, 0, 0x53053053) = 0
mprotect(0x56523a375000, 4096, PROT_READ) = 0
exit(0)                                 = ?
+++ exited with 0 +++
```

This is actually coming from dynamic linking. We don't have any symbols to
dynamically link (in fact we only have one symbol, `_start`, which doesn't even
attempt to follow the C calling convention). Let's instead link the program statically:

```
$ clang -nostdlib -static tiny.s -o tiny
$ ./tiny
.rwxr-xr-x 4,608 tchajed 31 May 16:58 tiny
$ strace ./tiny
execve("./tiny", ["./tiny"], 0x7ffe41ada7d0 /* 26 vars */) = 0
exit(0)                                 = ?
+++ exited with 0 +++
```

Great, we're down to 4.6KB. The file [tiny.s](tiny.s) just makes one change:
instead of doing `mov %60, %rax` like a normal person, we use `push` and `pop`
since these only take 3 bytes instead of 4.

## Manually writing a binary

The binary produced by using the normal assembler still has a lot of cruft:

```
$ objdump -x tiny_asm
tiny_asm:     file format elf64-x86-64
tiny_asm
architecture: i386:x86-64, flags 0x00000112:
EXEC_P, HAS_SYMS, D_PAGED
start address 0x0000000000401000

Program Header:
    LOAD off    0x0000000000000000 vaddr 0x0000000000400000 paddr 0x0000000000400000 align 2**12
         filesz 0x00000000000000b0 memsz 0x00000000000000b0 flags r--
    LOAD off    0x0000000000001000 vaddr 0x0000000000401000 paddr 0x0000000000401000 align 2**12
         filesz 0x0000000000000007 memsz 0x0000000000000007 flags r-x

Sections:
Idx Name          Size      VMA               LMA               File off  Algn
  0 .text         00000007  0000000000401000  0000000000401000  00001000  2**2
                  CONTENTS, ALLOC, LOAD, READONLY, CODE
SYMBOL TABLE:
0000000000401000 g       .text	0000000000000000 _start
0000000000402000 g       .text	0000000000000000 __bss_start
0000000000402000 g       .text	0000000000000000 _edata
0000000000402000 g       .text	0000000000000000 _end
```

There are all these symbols and sections that aren't needed. Even after `strip`
the file is still over 4KB.

To really optimize the binary, we can instead produce an ELF file completely
manually. This is where [this Rust program](src/main.rs), which constructs an
ELF binary by defining the appropriate binary structs and then writing them out.

The result is a remarkably small file:

```
$ cargo run
$ ./tiny
$ ll tiny
.rwxr-xr-x 127 tchajed 31 May 17:04 tiny
```

Just 127 bytes! And we wrote all of them manually (except for the 7-byte
program `push`, remember that?).
