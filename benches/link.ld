SECTIONS
{
  . = 0x0;
  .text : { *(.text) }
  .rodata : { *(.rodata) }
  . = ALIGN(4);
  . = 0x4000;
  .data : { *(.data) }
  .bss : { *(.bss) }
}
