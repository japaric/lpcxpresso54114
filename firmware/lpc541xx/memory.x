MEMORY
{
  /* NOTE FLASH is connected to a single bus */
  /* for core #0 */
  FLASH0 : ORIGIN = 0x00000000, LENGTH = 128K
  /* for core #1 */
  FLASH1 : ORIGIN = 0x00020000, LENGTH = 128K

  /* NOTE each region is connected a different AHB / APB bus */
  /* for core #0 */
  SRAM0  : ORIGIN = 0x20000000, LENGTH = 64K
  /* for core #1 */
  SRAM1  : ORIGIN = 0x20010000, LENGTH = 64K
  /* for `#[shared]` variables */
  SRAM2  : ORIGIN = 0x20020000, LENGTH = 32K
  SRAMX  : ORIGIN = 0x04000000, LENGTH = 32K
}
