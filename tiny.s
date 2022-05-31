.globl _start
.text

_start:
	pushq $60
	popq  %rax

	// mov $60, %ax
	// xorl %eax, %eax
	syscall
