.code16
.globl start
start:
    cli

    xorw    %ax, %ax
    movw    %ax, %ds
    movw    %ax, %es
    movw    %ax, %ss

set_a20_1:
    inb     $0x64, %al
    testb   $0x02, %al
    jnz     set_a20_1

    movb    $0xD1, %al
    outb    %al, $0x64

set_a20_2:
    inb     $0x64, %al
    testb   $0x02, %al
    jnz     set_a20_2

    movb    $0xDF, %al
    outb    %al, $0x60

    lgdt    gdtdesc
    movl    %cr0, %eax
    orl     $0x1, %eax      # enable protect mode
    movl    %eax, %cr0

    ljmp    $(1 << 3), $start32

.code32
start32:
    movw    $(2 << 3), %ax
    movw    %ax, %ds
    movw    %ax, %es
    movw    %ax, %ss
    movw    $00, %ax
    movw    %ax, %fs
    movw    %ax, %gs

    movl    $start, %esp

    call    bootmain        # call rust function

    movw    $0x8A00, %ax
    movw    %ax, %dx
    outw    %ax, %dx
    movw    $0x8AE0, %ax
    outw    %ax, %dx
spin:
    hlt
    jmp     spin

.p2align  2
gdt:
    # null descriptor
    .word   0x000, 0x0000
    .byte   0x00, 0x00, 0x00, 0x00

    # kernel code
    .word   0xFFFF, 0x0000
    .byte   0x00, 0x9A, 0xCF, 0x00

    # kernel data+stack
    .word   0xFFFF, 0x0000
    .byte   0x00, 0x92, 0xCF, 0x00
gdtdesc:
    .word   (gdtdesc - gdt - 1)
    .long   gdt