OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    etext = .;

    . = ALIGN(4K);
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
    }
    erodata = .;

    . = ALIGN(4K);
    sdata = .;
    .data : {
        *(.data .data.*)
    }
    edata = .;

    . = ALIGN(4K);
    sbss = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
    }
    ebss = .;
    ekernel = .;

    . = ALIGN(4K);
    frames = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
