.section .smp_trampoline, "awx"
.global _smp_trampoline
.intel_syntax noprefix
.align 4096
.code16

_smp_trampoline:
    cli

trampoline_gdt:
    
    .quad 0

# __KERNEL_CS
	.word 0xFFFF
	.word 0
	.byte 0
	.byte 0b10011010
	.byte 0b11001111
	.byte 0

	# __KNLUSR_DS
	.word 0xFFFF
	.word 0
	.byte 0
	.byte 0b10010010
	.byte 0b11001111
	.byte 0
trampoline_gdtDesc:
	.word (trampoline_gdtDesc - trampoline_gdt) - 1
	.quad trampoline_gdt
