OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS {
  . = 0x80200000;
  .text : {
    *(.text.boot)
    *(.text .text.*)
  }
  .rodata : {
    *(.rodata .rodata.*)
  }
  .data : {
    *(.data .data.*)
  }
  .bss : {
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
