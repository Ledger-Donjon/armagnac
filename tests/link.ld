SECTIONS
{
  . = 0x0;
  .text :
  {
      *(.vectors)
      *(.text)
  }
  .rodata : { *(.rodata) }
  . = ALIGN(4);
  . = 0x8000;
  .data : { *(.data) }
  .bss : { *(.bss) }
}
