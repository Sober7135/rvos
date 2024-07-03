ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
  . = BASE_ADDRESS;
  stext = .;
  .text : { 
      *(.text.entry) 
      . = ALIGN(4K);
      strampoline = .;
      *(.text.trampoline)
      . = ALIGN(4K);
      *(.text .text.*) 
  }
  . = ALIGN(4K);
  etext = .;

  . = ALIGN(4K);
  srodata = .;
  .rodata : { *(.rodata) *(.rodata.*)}
  . = ALIGN(4K);
  erodata = .;
  
  . = ALIGN(4K);
  sdata = .;
  .data : { *(.data) *(.sdata) }
  . = ALIGN(4K);
  edata = .;

  . = ALIGN(4K);
  sbss = .;
  .bss : { *(.bss .bss.*) *(.bss.stack) }
  . = ALIGN(4K);
  ebss = .;

  ekernel = .;

  /DISCARD/ : {
      *(.eh_frame)
      # *(.debug*)
  }
}