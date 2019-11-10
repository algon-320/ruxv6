.set KERNBASE,      0x80000000  # First kernel virtual address
.set CR0_WP,        0x00010000  # Write Protect
.set CR0_PG,        0x80000000  # Paging
.set CR4_PSE,       0x00000010  # Page size extension

.set KSTACKSIZE,    4096 * 2    # Size of per-process kernel stack

.p2align 2
.text

# TODO support multiboot

.globl _start
_start = (entry - KERNBASE)

.globl entry
entry:
    # Turn on page size extension for 4Mbyte pages
    movl    %cr4, %eax
    orl     $(CR4_PSE), %eax
    movl    %eax, %cr4

    # Set page directory
    movl    $(entrypgdir - KERNBASE), %eax
    movl    %eax, %cr3

    # Turn on paging
    movl    %cr0, %eax
    orl     $(CR0_PG|CR0_WP), %eax
    movl    %eax, %cr0

    # Set up the stack pointer
    movl    $(stack + KSTACKSIZE), %esp

    # Jump to main()
    mov     $main, %eax
    jmp     *%eax

.comm stack, KSTACKSIZE