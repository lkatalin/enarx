.text
	.globl _start
	.type _start, @function

_start:
        mov $228,   	%rax /* SYS_clock_gettime */
        mov $1,    	%edi /* clock_id = libc::CLOCK_MONOTONIC */
 	lea ts(%rip),   %rsi /* timespec */
        syscall

        mov $60,   	%rax /* SYS_exit */
        mov $0,    	%rdi /* exit status */
        syscall

.data
ts:
	.space 128
