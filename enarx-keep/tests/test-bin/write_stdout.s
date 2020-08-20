	.text
	.globl _start
	.type _start, @function

.LC0:
        .string "hi\n"

_start:
	mov $1,    	%rax /* SYS_write */
	mov $1,    	%rdi /* STDOUT_FILENO */
	lea .LC0(%rip), %rsi
	mov $3,    	%rdx /* length */
	syscall

	mov $60,  	 %rax /* SYS_exit */
	mov $0,   	 %rdi /* exit status */
	syscall
