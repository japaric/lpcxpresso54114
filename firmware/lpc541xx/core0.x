INCLUDE memory.x;

ENTRY(_start);
EXTERN(VECTORS);

_stack_top = ORIGIN(SRAM0) + LENGTH(SRAM0);

SECTIONS
{
  .vectors :
  {
    LONG(_stack_top);
    LONG(_start);

    KEEP(*(.vectors));
  } > FLASH0

  .text : ALIGN(4)
  {
    *(.text .text.*);
  } > FLASH0

  .rodata : ALIGN(4)
  {
    *(.rodata .rodata.*);

    . = ALIGN(4);
  } > FLASH0

  .bss : ALIGN(4)
  {
    *(.bss .bss.*);

    . = ALIGN(4);
  } > SRAM0

  _sbss = ADDR(.bss);
  _ebss = ADDR(.bss) + SIZEOF(.bss);

  .data : ALIGN(4)
  {
    *(.data .data.*);

    . = ALIGN(4);
  } > SRAM0 AT > FLASH0

  _sdata = ADDR(.data);
  _edata = ADDR(.data) + SIZEOF(.data);
  _sidata = LOADADDR(.data);

  .shared : ALIGN(4)
  {
    KEEP(microamp-data.o(.shared));
    . = ALIGN(4);
  } > SRAM2 AT > FLASH0

  _sshared = ADDR(.shared);
  _eshared = ADDR(.shared) + SIZEOF(.shared);
  _sishared = LOADADDR(.shared);

  /DISCARD/ :
  {
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}

PROVIDE(NMI = DefaultHandler);
PROVIDE(HardFault = DefaultHandler);
PROVIDE(MemoryManagement = DefaultHandler);
PROVIDE(BusFault = DefaultHandler);
PROVIDE(UsageFault = DefaultHandler);
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
PROVIDE(PIN_INT4 = DefaultHandler);
PROVIDE(PIN_INT5 = DefaultHandler);
PROVIDE(PIN_INT6 = DefaultHandler);
PROVIDE(PIN_INT7 = DefaultHandler);
PROVIDE(CTIMER2 = DefaultHandler);
PROVIDE(CTIMER4 = DefaultHandler);
