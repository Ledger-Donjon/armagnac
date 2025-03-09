SECTIONS
{
  . = 0x0;
  .text : { *(.text) }
  .rodata : { *(.rodata) }
  . = ALIGN(4);
  . = 0x400;
  .data : { *(.data) }
  .bss : { *(.bss) }
}
