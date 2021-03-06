INCLUDE memory.x;

ENTRY(start);
EXTERN(VECTORS);

_stack_top = ORIGIN(SRAM1) + LENGTH(SRAM1);

SECTIONS
{
  .pointers :
  {
    LONG(ADDR(.vectors));
    LONG(ADDR(.data) + SIZEOF(.data));
  } > FLASH1

  .vectors :
  {
    LONG(_stack_top);
    LONG(start);

    KEEP(*(.vectors));
  } > SRAM1 AT > FLASH1

  .text : ALIGN(4)
  {
    *(.text .text.*);

    . = ALIGN(4);
  } > SRAM1 AT > FLASH1

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);

    . = ALIGN(4);
  } > SRAM1 AT > FLASH1

  .data : ALIGN(4)
  {
    *(.text .text.*);
    . = ALIGN(4);

    *(.data .data.*);

    . = ALIGN(4);
  } > SRAM1 AT > FLASH1

  .bss : ALIGN(4)
  {
    *(.bss .bss.*);

    . = ALIGN(4);
  } > SRAM1

  _sbss = ADDR(.bss);
  _ebss = ADDR(.bss) + SIZEOF(.bss);

  .shared (NOLOAD) : ALIGN(4)
  {
    KEEP(microamp-data.o(.shared));
    . = ALIGN(4);
  } > SRAM2

  /DISCARD/ :
  {
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}

PROVIDE(NMI = DefaultHandler);
PROVIDE(HardFault = DefaultHandler);
/* PROVIDE(MemoryManagement = DefaultHandler); */
/* PROVIDE(BusFault = DefaultHandler); */
/* PROVIDE(UsageFault = DefaultHandler); */
PROVIDE(SVCall = DefaultHandler);
PROVIDE(DebugMonitor = DefaultHandler);
PROVIDE(PendSV = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);
PROVIDE(WDT = DefaultHandler);
PROVIDE(DMA = DefaultHandler);
PROVIDE(GINT0 = DefaultHandler);
PROVIDE(GINT1 = DefaultHandler);
PROVIDE(PIN_INT0 = DefaultHandler);
PROVIDE(PIN_INT1 = DefaultHandler);
PROVIDE(PIN_INT2 = DefaultHandler);
PROVIDE(PIN_INT3 = DefaultHandler);
PROVIDE(UTICK = DefaultHandler);
PROVIDE(MRT = DefaultHandler);
PROVIDE(CTIMER0 = DefaultHandler);
PROVIDE(CTIMER1 = DefaultHandler);
PROVIDE(SCT0 = DefaultHandler);
PROVIDE(CTIMER3 = DefaultHandler);
PROVIDE(Flexcomm0 = DefaultHandler);
PROVIDE(Flexcomm1 = DefaultHandler);
PROVIDE(Flexcomm2 = DefaultHandler);
PROVIDE(Flexcomm3 = DefaultHandler);
PROVIDE(Flexcomm4 = DefaultHandler);
PROVIDE(Flexcomm5 = DefaultHandler);
PROVIDE(Flexcomm6 = DefaultHandler);
PROVIDE(Flexcomm7 = DefaultHandler);
PROVIDE(ADC0_SEQA = DefaultHandler);
PROVIDE(ADC0_SEQB = DefaultHandler);
PROVIDE(ADC0_THCMP = DefaultHandler);
PROVIDE(DMIC = DefaultHandler);
PROVIDE(HWVAD = DefaultHandler);
PROVIDE(USB_WAKEUP = DefaultHandler);
PROVIDE(USB = DefaultHandler);
PROVIDE(RTC = DefaultHandler);
PROVIDE(MAILBOX = DefaultHandler);
/* PROVIDE(PIN_INT4 = DefaultHandler); */
/* PROVIDE(PIN_INT5 = DefaultHandler); */
/* PROVIDE(PIN_INT6 = DefaultHandler); */
/* PROVIDE(PIN_INT7 = DefaultHandler); */
/* PROVIDE(CTIMER2 = DefaultHandler); */
/* PROVIDE(CTIMER4 = DefaultHandler); */

ASSERT(ADDR(.pointers) == ORIGIN(FLASH1), ".pointers is not positioned where expected");

/* check that all sections that need to be initialized are contiguous */
ASSERT(ADDR(.vectors) + SIZEOF(.vectors) == ADDR(.text),
".vectors and .text are not contiguous");
ASSERT(ADDR(.text) + SIZEOF(.text) == ADDR(.rodata),
".text and .rodata are not contiguous");
ASSERT(ADDR(.rodata) + SIZEOF(.rodata) == ADDR(.data),
".rodata and .data are not contiguous");

ASSERT(LOADADDR(.vectors) + SIZEOF(.vectors) == LOADADDR(.text),
".vectors and .text are not contiguous");
ASSERT(LOADADDR(.text) + SIZEOF(.text) == LOADADDR(.rodata),
".text and .rodata are not contiguous");
ASSERT(LOADADDR(.rodata) + SIZEOF(.rodata) == LOADADDR(.data),
".rodata and .data are not contiguous");
