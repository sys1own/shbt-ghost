/*
 * linker.ld — SHBT-GHOST bare-metal memory map.
 *
 * The .stinespring_frame section carves a 2,112-byte UnifiedStinespringFrame
 * SRAM arena on a 64-byte boundary: 640 B active residual (eta_A = 10/33)
 * followed by 1,472 B dark ledger QEC storage (eta_D = 23/33).
 */
OUTPUT_FORMAT("elf64-littleaarch64", "elf64-littleaarch64", "elf64-littleaarch64")
OUTPUT_ARCH(aarch64)
ENTRY(_start)

MEMORY
{
    ROM   (rx)  : ORIGIN = 0x00000000, LENGTH = 512K
    MMIO  (rw)  : ORIGIN = 0x70000000, LENGTH = 56
    SRAM  (rwx) : ORIGIN = 0x20000000, LENGTH = 128K
}

SECTIONS
{
    .text : {
        *(.text._start)
        *(.text*)
    } > ROM

    .rodata : { *(.rodata*) } > ROM

    /* 2,112-byte Stinespring arena, 64-byte aligned. */
    .stinespring_frame : ALIGN(64) {
        __stinespring_frame_start = .;
        *(.stinespring_frame)
        . = __stinespring_frame_start + 2112;
        __stinespring_frame_end = .;
    } > SRAM

    .data : { *(.data*) } > SRAM
    .bss : {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    } > SRAM

    .shbt_mmio (NOLOAD) : {
        . = . + 56;
    } > MMIO
}
