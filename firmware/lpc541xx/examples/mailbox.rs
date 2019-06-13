#![no_main]
#![no_std]

use core::sync::atomic::{self, AtomicBool, Ordering};

use lpc541xx as _;
use panic_halt as _;

#[microamp::shared]
static BARRIER: AtomicBool = AtomicBool::new(false);

#[no_mangle]
unsafe extern "C" fn main() -> ! {
    if cfg!(core = "0") {
        const NVIC_ISER: *mut u32 = 0xE000_E100 as *mut u32;

        // unmask GINT0
        NVIC_ISER.write_volatile(1 << 2);

        // unblock core #1
        BARRIER.store(true, Ordering::Release);
    }

    if cfg!(core = "1") {
        const MAILBOX: usize = 0x4008_B000;
        const MAILBOX_IRQ1SET: *mut u32 = (MAILBOX + 0x14) as *mut u32;

        while !BARRIER.load(Ordering::Acquire) {}

        // trigger core #0 interrupt #2 (GINT0)
        MAILBOX_IRQ1SET.write_volatile(1 << 2);
    }

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[cfg(core = "0")]
#[no_mangle]
#[allow(non_snake_case)]
unsafe fn GINT0() {
    use cortex_m::{iprintln, peripheral::ITM};

    let mut itm = core::mem::transmute::<_, ITM>(());

    iprintln!(&mut itm.stim[0], "GINT0");
}
