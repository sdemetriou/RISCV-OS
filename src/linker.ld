OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS {
  . = 0x80200000;
  .text : {
    . = ALIGN(16);
    *(.text.boot)
    *(.text .text.*)
  }
  .rodata : {
    . = ALIGN(16);
    *(.rodata .rodata.*)
  }
  .data : {
    . = ALIGN(16);
    *(.data .data.*)
  }
  .bss : {
    . = ALIGN(16);
    start_bss = .;
    *(.bss .bss.*)
    end_bss = .;
  }
  .heap : {
    . = ALIGN(16);
    b_hp = .;
    . += 0x1000;
    t_hp = .;
  }
  .stack : {
    . = ALIGN(16);
    b_sp = .;
    . += 0x1000;
    t_sp = .;
  }
  .procheap : {
    . = ALIGN(16);
    b_php = .;
    . += 0x1000;
    t_php = .;
  }
  .procstack : {
    . = ALIGN(16);
    b_psp = .;
    . += 0x1000;
    t_psp = .;
  }
}
