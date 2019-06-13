  # LLD requires that the section flags are explicitly set here
  .section .text._start, "ax"
  .global _start
  .syntax unified
  # .type and .thumb_func are both required; otherwise its Thumb bit does not
  # get set and an invalid vector table is generated
  .type _start,%function
  .thumb_func
_start:
  #  read CPUID
  ldr   r0, =0x0e000ed00
  ldr   r0, [r0, #0]
  ldr   r1, =0x0000fff0
  ands  r1, r0
  ldr   r0, =0x0000c240
  cmp   r1, r0
  beq   2f

  #  Cortex-M0+; read CPBOOT
  ldr   r1, =0x40000804
  ldr   r0, [r1, #0]
  cmp   r0, #0
  bne   3f

  # CPBOOT=0; wait for a reset
1:
  wfi
  b     1b

  #  Cortex-M4F
2:
  ldr  r0, =start
  bx   r0

3:
  # read CPSTACK
  ldr   r1, [r1, #4]
  mov   sp, r1
  bx    r0
