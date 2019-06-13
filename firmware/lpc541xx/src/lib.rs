#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(warnings)]
#![no_std]

use core::{
    cmp,
    convert::{Infallible, TryInto},
    fmt, ops,
    sync::atomic::{self, Ordering},
};

use bare_metal::Nr;
use rtfm::Monotonic;

#[cfg(master)]
pub const NVIC_PRIO_BITS: u8 = 3;

#[cfg(not(master))]
pub const NVIC_PRIO_BITS: u8 = 2;

const MAILBOX_BASE: usize = 0x4008_B000;

pub struct CTIMER0;

const CTIMER0_BASE: usize = 0x40008000;

const CTIMER0_TCR: *mut u32 = (CTIMER0_BASE + 0x4) as *mut u32;
const CTIMER0_TC: *const u32 = (CTIMER0_BASE + 0x8) as *const u32;

unsafe impl Monotonic for CTIMER0 {
    type Instant = Instant;

    fn ratio() -> u32 {
        1
    }

    fn now() -> Instant {
        Instant::now()
    }

    /// Resets the counter to *zero*
    unsafe fn reset() {
        CTIMER0_TCR.write_volatile(0b01); // release from reset
    }

    fn zero() -> Instant {
        Instant { inner: 0 }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Instant {
    inner: i32,
}

impl Instant {
    pub fn now() -> Self {
        Self {
            inner: unsafe { CTIMER0_TC.read_volatile() } as i32,
        }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        let diff = self.inner.wrapping_sub(earlier.inner);
        assert!(diff >= 0, "second instant is later than `self`");
        Duration { inner: diff as u32 }
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Instant")
            .field(&(self.inner as u32))
            .finish()
    }
}

impl ops::Add<Duration> for Instant {
    type Output = Self;

    fn add(self, dur: Duration) -> Instant {
        Instant {
            inner: self.inner.wrapping_add(dur.inner as i32),
        }
    }
}

impl ops::Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        self.duration_since(rhs)
    }
}

impl Ord for Instant {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        self.inner.wrapping_sub(rhs.inner).cmp(&0)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

#[derive(Clone, Copy)]
pub struct Duration {
    inner: u32,
}

impl Duration {
    pub fn from_cycles(cycles: u32) -> Self {
        Self { inner: cycles }
    }

    pub fn as_cycles(&self) -> u32 {
        self.inner
    }
}

impl TryInto<u32> for Duration {
    type Error = Infallible;

    fn try_into(self) -> Result<u32, Infallible> {
        Ok(self.inner)
    }
}

pub fn xpend(core: u8, int: impl Nr) {
    // Cortex-M0+
    const MAILBOX_IRQ0SET: *mut u32 = (MAILBOX_BASE + 0x04) as *mut u32;

    // Cortex-M4
    const MAILBOX_IRQ1SET: *mut u32 = (MAILBOX_BASE + 0x14) as *mut u32;

    let nr = int.nr();

    assert!(core < 2);
    assert!(nr < 32);

    unsafe {
        if core == 0 {
            MAILBOX_IRQ1SET.write_volatile(1 << nr);
        } else {
            MAILBOX_IRQ0SET.write_volatile(1 << nr);
        }
    }
}

// forward interrupts
#[no_mangle]
extern "C" fn MAILBOX() {
    unsafe {
        const NVIC_ISPR: *mut u32 = 0xE000_E200 as *mut u32;

        match () {
            #[cfg(master)]
            () => {
                // Cortex-M4
                const MAILBOX_IRQ1: *const u32 = (MAILBOX_BASE + 0x10) as *const u32;
                const MAILBOX_IRQ1CLR: *mut u32 = (MAILBOX_BASE + 0x18) as *mut u32;

                let mask = MAILBOX_IRQ1.read_volatile();
                NVIC_ISPR.write_volatile(mask);
                MAILBOX_IRQ1CLR.write_volatile(mask);
            }

            #[cfg(not(master))]
            () => {
                // Cortex-M0+
                const MAILBOX_IRQ0: *const u32 = MAILBOX_BASE as *const u32;
                const MAILBOX_IRQ0CLR: *mut u32 = (MAILBOX_BASE + 0x8) as *mut u32;

                let mask = MAILBOX_IRQ0.read_volatile();
                NVIC_ISPR.write_volatile(mask);
                MAILBOX_IRQ0CLR.write_volatile(mask);
            }
        }
    }
}

// This is the pseudo-Rust version of the common entry point, executed by both cores.
//
// Because this requires a custom calling convention this is actually written in assembly and can
// be found in the `asm.s` file
#[cfg(unused)]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    const CPUID: *mut u32 = 0x0e000_ed00 as *mut u32;
    const SYSCON_CPBOOT: *mut u32 = 0x4000_0804 as *mut u32;
    const SYSCON_CPSTACK: *mut u32 = 0x4000_0808 as *mut u32;

    if (CPUID.read_volatile() >> 4) & 0xfff == 0xc24 {
        // this is the Cortex-M4F core
        start()
    } else {
        // this is the Cortex-M0+ core
        let boot = SYSCON_CPBOOT.read_volatile();
        if boot == 0 {
            // not yet set by the master; sleep
            loop {
                // WFI
            }
        } else {
            // written to the SP (Stack Pointer) register
            let _sp = SYSCON_CPSTACK.read_volatile();
            let boot = core::mem::transmute::<u32, extern "C" fn() -> !>(boot);
            // NOTE "branch" not "branch link"
            boot()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn start() -> ! {
    // Start of core #1 (slave) vector table
    // XXX It'd be best to get this from the linker script
    const SLAVE_VECTORS: *const u32 = 0x0002_0000 as *const u32;

    match () {
        #[cfg(master)]
        () => {
            extern "C" {
                static mut _sshared: u32;
                static mut _eshared: u32;
                static _sishared: u32;
            }

            const SYSCON_AHBCLKCTRLSET0: *mut u32 = 0x4000_0220 as *mut u32;

            // enable SRAM1, SRAM2 and MAILBOX
            // (SRAM1 should be enabled on boot / reset according to the data sheet but it isn't)
            SYSCON_AHBCLKCTRLSET0.write_volatile((1 << 26) | (1 << 4) | (1 << 3));

            const SYSCON_AHBCLKCTRLSET1: *mut u32 = 0x4000_0224 as *mut u32;

            // enable CTIMER0
            SYSCON_AHBCLKCTRLSET1.write_volatile(1 << 26);

            // held the CTIMER0 counter in reset
            CTIMER0_TCR.write_volatile(0b10);

            // SRAM2 must be enabled before the `.shared` section is initialized
            atomic::compiler_fence(Ordering::SeqCst);

            // initialize `.shared` variables
            // this is *not* a volatile operation so compiler fences are required
            r0::init_data(&mut _sshared, &mut _eshared, &_sishared);

            // ensure that the slave is only active *after* shared variables have been initialized
            atomic::compiler_fence(Ordering::SeqCst);

            const SYSCON_CPBOOT: *mut u32 = 0x4000_0804 as *mut u32;
            const SYSCON_CPSTACK: *mut u32 = 0x4000_0808 as *mut u32;
            const SYSCON_CPUCTRL: *mut u32 = 0x4000_0800 as *mut u32;

            SYSCON_CPSTACK.write_volatile(SLAVE_VECTORS.read());
            SYSCON_CPBOOT.write_volatile(SLAVE_VECTORS.add(1).read());

            // enable the M0+ clock but hold the core in reset
            SYSCON_CPUCTRL.write_volatile(0xc0c4_806d);

            // release the M0+ from reset
            SYSCON_CPUCTRL.write_volatile(0xc0c4_804d);
        }

        #[cfg(not(master))]
        () => {
            const SCB_VTOR: *mut u32 = 0xe000_ed08 as *mut u32;

            // after reset the slave uses 0x0 as the start of the vector table
            // this needs to be updated to use the right address
            SCB_VTOR.write_volatile(SLAVE_VECTORS as u32);
        }
    }

    const NVIC_ISER: *mut u32 = 0xE000_E100 as *mut u32;

    // unmask the MAILBOX interrupt
    NVIC_ISER.write_volatile(1 << 31);

    extern "C" {
        static mut _sbss: u32;
        static mut _ebss: u32;

        static mut _sdata: u32;
        static mut _edata: u32;
        static _sidata: u32;
    }

    // initialize .bss and .data
    r0::zero_bss(&mut _sbss, &mut _ebss);
    r0::init_data(&mut _sdata, &mut _edata, &_sidata);

    // do not run `main` before the `static` variables have been initialized
    atomic::compiler_fence(Ordering::SeqCst);

    extern "Rust" {
        // user program entry point
        fn main() -> !;
    }

    match () {
        #[cfg(master)]
        () => {
            const SCB_CPACR: *mut u32 = 0xe000_ed88 as *mut u32;

            // enable the FPU
            SCB_CPACR.write_volatile(SCB_CPACR.read_volatile() | (0b01_01 << 20) | 0b10_10 << 20);

            // trampoline to prevent the user `main` from being inlined into this (`start`) function
            #[inline(never)]
            #[no_mangle]
            unsafe fn _main() -> ! {
                main()
            }

            _main()
        }

        #[cfg(not(master))]
        () => main(),
    }
}

#[no_mangle]
extern "C" fn DefaultHandler(_msp: &ExceptionFrame) -> ! {
    loop {}
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ExceptionFrame {
    /// (General purpose) Register 0
    pub r0: u32,

    /// (General purpose) Register 1
    pub r1: u32,

    /// (General purpose) Register 2
    pub r2: u32,

    /// (General purpose) Register 3
    pub r3: u32,

    /// (General purpose) Register 12
    pub r12: u32,

    /// Linker Register
    pub lr: u32,

    /// Program Counter
    pub pc: u32,

    /// Program Status Register
    pub xpsr: u32,
}

#[allow(non_camel_case_types)]
pub enum Interrupt {
    WDT,
    DMA,
    GINT0,
    GINT1,
    PIN_INT0,
    PIN_INT1,
    PIN_INT2,
    PIN_INT3,
    UTICK,
    MRT,
    CTIMER0,
    CTIMER1,
    SCT0,
    CTIMER3,
    Flexcomm0,
    Flexcomm1,
    Flexcomm2,
    Flexcomm3,
    Flexcomm4,
    Flexcomm5,
    Flexcomm6,
    Flexcomm7,
    ADC0_SEQA,
    ADC0_SEQB,
    ADC0_THCMP,
    DMIC,
    HWVAD,
    USB_WAKEUP,
    USB,
    RTC,
    MAILBOX,
    #[cfg(master)]
    PIN_INT4,
    #[cfg(master)]
    PIN_INT5,
    #[cfg(master)]
    PIN_INT6,
    #[cfg(master)]
    PIN_INT7,
    #[cfg(master)]
    CTIMER2,
    #[cfg(master)]
    CTIMER4,
}

unsafe impl Nr for Interrupt {
    fn nr(&self) -> u8 {
        use Interrupt::*;

        match self {
            WDT => 0,
            DMA => 1,
            GINT0 => 2,
            GINT1 => 3,
            PIN_INT0 => 4,
            PIN_INT1 => 5,
            PIN_INT2 => 6,
            PIN_INT3 => 7,
            UTICK => 8,
            MRT => 9,
            CTIMER0 => 10,
            CTIMER1 => 11,
            SCT0 => 12,
            CTIMER3 => 13,
            Flexcomm0 => 14,
            Flexcomm1 => 15,
            Flexcomm2 => 16,
            Flexcomm3 => 17,
            Flexcomm4 => 18,
            Flexcomm5 => 19,
            Flexcomm6 => 20,
            Flexcomm7 => 21,
            ADC0_SEQA => 22,
            ADC0_SEQB => 23,
            ADC0_THCMP => 24,
            DMIC => 25,
            HWVAD => 26,
            USB_WAKEUP => 27,
            USB => 28,
            RTC => 29,
            MAILBOX => 31,
            #[cfg(master)]
            PIN_INT4 => 32,
            #[cfg(master)]
            PIN_INT5 => 33,
            #[cfg(master)]
            PIN_INT6 => 34,
            #[cfg(master)]
            PIN_INT7 => 35,
            #[cfg(master)]
            CTIMER2 => 36,
            #[cfg(master)]
            CTIMER4 => 37,
        }
    }
}

extern "C" {
    fn NMI();
    fn HardFault();
    #[cfg(master)]
    fn MemoryManagement();
    #[cfg(master)]
    fn BusFault();
    #[cfg(master)]
    fn UsageFault();
    fn SVCall();
    fn DebugMonitor();
    fn PendSV();
    fn SysTick();
    fn WDT();
    fn DMA();
    fn GINT0();
    fn GINT1();
    fn PIN_INT0();
    fn PIN_INT1();
    fn PIN_INT2();
    fn PIN_INT3();
    fn UTICK();
    fn MRT();
    fn CTIMER1();
    fn SCT0();
    fn CTIMER3();
    fn Flexcomm0();
    fn Flexcomm1();
    fn Flexcomm2();
    fn Flexcomm3();
    fn Flexcomm4();
    fn Flexcomm5();
    fn Flexcomm6();
    fn Flexcomm7();
    fn ADC0_SEQA();
    fn ADC0_SEQB();
    fn ADC0_THCMP();
    fn DMIC();
    fn HWVAD();
    fn USB_WAKEUP();
    fn USB();
    fn RTC();
    // defined above
    // fn MAILBOX();
    #[cfg(master)]
    fn PIN_INT4();
    #[cfg(master)]
    fn PIN_INT5();
    #[cfg(master)]
    fn PIN_INT6();
    #[cfg(master)]
    fn PIN_INT7();
    #[cfg(master)]
    fn CTIMER2();
    #[cfg(master)]
    fn CTIMER4();
}

#[cfg(master)]
#[link_section = ".vectors"]
#[no_mangle]
static VECTORS: [Option<unsafe extern "C" fn()>; 52] = [
    // Cortex-M exceptions
    Some(NMI),
    Some(HardFault),
    Some(MemoryManagement),
    Some(BusFault),
    Some(UsageFault),
    None, // NOTE checksum goes here
    None,
    None,
    None,
    Some(SVCall),
    Some(DebugMonitor),
    None,
    Some(PendSV),
    Some(SysTick),
    // 0 - Device-specific interrupts
    Some(WDT),
    Some(DMA),
    Some(GINT0),
    Some(GINT1),
    Some(PIN_INT0),
    Some(PIN_INT1),
    Some(PIN_INT2),
    Some(PIN_INT3),
    Some(UTICK),
    Some(MRT),
    // 10
    Some({
        extern "C" {
            fn CTIMER0();
        }

        CTIMER0
    }),
    Some(CTIMER1),
    Some(SCT0),
    Some(CTIMER3),
    Some(Flexcomm0),
    Some(Flexcomm1),
    Some(Flexcomm2),
    Some(Flexcomm3),
    Some(Flexcomm4),
    Some(Flexcomm5),
    // 20
    Some(Flexcomm6),
    Some(Flexcomm7),
    Some(ADC0_SEQA),
    Some(ADC0_SEQB),
    Some(ADC0_THCMP),
    Some(DMIC),
    Some(HWVAD),
    Some(USB_WAKEUP),
    Some(USB),
    Some(RTC),
    // 30
    None,
    Some(MAILBOX),
    Some(PIN_INT4),
    Some(PIN_INT5),
    Some(PIN_INT6),
    Some(PIN_INT7),
    Some(CTIMER2),
    Some(CTIMER4),
];

#[cfg(not(master))]
#[link_section = ".vectors"]
#[no_mangle]
static VECTORS: [Option<unsafe extern "C" fn()>; 46] = [
    // Cortex-M exceptions
    Some(NMI),
    Some(HardFault),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(SVCall),
    Some(DebugMonitor),
    None,
    Some(PendSV),
    Some(SysTick),
    // 0 - Device-specific interrupts
    Some(WDT),
    Some(DMA),
    Some(GINT0),
    Some(GINT1),
    Some(PIN_INT0),
    Some(PIN_INT1),
    Some(PIN_INT2),
    Some(PIN_INT3),
    Some(UTICK),
    Some(MRT),
    // 10
    Some({
        extern "C" {
            fn CTIMER0();
        }

        CTIMER0
    }),
    Some(CTIMER1),
    Some(SCT0),
    Some(CTIMER3),
    Some(Flexcomm0),
    Some(Flexcomm1),
    Some(Flexcomm2),
    Some(Flexcomm3),
    Some(Flexcomm4),
    Some(Flexcomm5),
    // 20
    Some(Flexcomm6),
    Some(Flexcomm7),
    Some(ADC0_SEQA),
    Some(ADC0_SEQB),
    Some(ADC0_THCMP),
    Some(DMIC),
    Some(HWVAD),
    Some(USB_WAKEUP),
    Some(USB),
    Some(RTC),
    // 30
    None,
    Some(MAILBOX),
];
