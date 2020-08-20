	.text
	.globl _start
	.type _start, @function

_start:
        mov $60,   %rax /* SYS_exit */
        mov $1,    %rdi /* exit status */
        syscall
