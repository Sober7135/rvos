ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
  . = BASE_ADDRESS;
  stext = .;
  .text : { *(.text.entry) *(.text) *(.text.*) }
  . = ALIGN(4K);
  etext = .;

  srodata = .;
  .rodata : { *(.rodata) *(.rodata.*)}
  . = ALIGN(4K);
  erodata = .;
  
  sdata = .;
  .data : { *(.data) }
  . = ALIGN(4K);
  edata = .;

  sbss = .;
  .bss : { *(.bss) *(.bss.stack) }
  . = ALIGN(4K);
  ebss = .;
}