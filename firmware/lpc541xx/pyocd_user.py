def did_reset(core):
    CPUID = 0xE000ED00

    if (core.read_memory(CPUID) >> 4) & 0xfff == 0xc24:
        # Cortex-M4F (master)

        PIO0_15 = 0x4000103c

        SYSCON_AHBCLKCTRLSET0 = 0x40000220
        SYSCON_TRACECLKDIV = 0x40000304

        DCB_DEMCR = 0xE000EDFC

        TPIU_ACPR = 0xE0040010
        TPIU_FFCR = 0xE0040304
        TPIU_SPPR = 0xE00400F0

        ITM_LAR = 0xE0040FB0
        ITM_TCR = 0xE0000E80
        ITM_TER0 = 0xE0000E00

        # enable the IOCON peripheral
        core.write_memory(SYSCON_AHBCLKCTRLSET0, 1 << 13)

        # default clock is 12 MHz; set SWO divider to 6
        core.write_memory(SYSCON_TRACECLKDIV, 5)

        # configure PIO0_15 as SWO
        core.write_memory(PIO0_15, 0x182)

        # enable TPIU and ITM
        core.write_memory(DCB_DEMCR, core.read_memory(DCB_DEMCR) | (1 << 24))

        # prescaler = 0
        core.write_memory(TPIU_ACPR, 0)

        # mode = SWO NRZ
        core.write_memory(TPIU_SPPR, 2)

        core.write_memory(TPIU_FFCR, core.read_memory(TPIU_FFCR) & ~(1 << 1))

        # unlock the ITM
        core.write_memory(ITM_LAR, 0xC5ACCE55)

        # configure the ITM
        # TraceBusID | SWOEN | ITMEN
        core.write_memory(ITM_TCR, (1 << 16) | (1 << 3) | (1 << 0))

        # enable stimulus port 0
        core.write_memory(ITM_TER0, 1)

        # print('Cortex-M4F: ITM enabled')
