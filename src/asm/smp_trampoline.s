.section .smp_trampoline, "awx"
.global _smp_trampoline
.intel_syntax noprefix
.align 4096
.code16

_smp_trampoline:
    cli
    mov sp, 0x7c00
    lgdt [gdt32info_trampoline]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    push 0x8
    mov eax, offset appm
    push eax
    retf

.code32
appm:
    lgdt [gdt_64_pointer]
    mov eax, offset _p4
    mov cr3, eax
    # enable PAE-flag in cr4 (Physical Address Extension)
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    # set the long mode bit in the EFER MSR (model specific register)
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    # enable paging in the cr0 register
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    push 0x8
    mov eax, offset kernel_ap_entry_trampoline # replace with user-specified entry point
    push eax
    retf




gdt32info_trampoline:
   .word gdt32_end_trampoline - gdt32_trampoline - 1  # last byte in table
   .word gdt32_trampoline                  # start of table

gdt32_trampoline:
    # entry 0 is always unused
    .quad 0
codedesc_trampoline:
    .byte 0xff
    .byte 0xff
    .byte 0
    .byte 0
    .byte 0
    .byte 0x9a
    .byte 0xcf
    .byte 0
datadesc_trampoline:
    .byte 0xff
    .byte 0xff
    .byte 0
    .byte 0
    .byte 0
    .byte 0x92
    .byte 0xcf
    .byte 0
gdt32_end_trampoline:
