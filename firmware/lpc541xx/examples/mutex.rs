#![no_main]
#![no_std]

use core::sync::atomic::{self, Ordering};

use lpc541xx as _;
use microamp::shared;
use panic_halt as _;

// non-atomic variable
#[shared] // <- means: same memory location on all the cores
static mut SHARED: u64 = 0;

// used to synchronize access to `SHARED`; this is a memory mapped register
const MAILBOX_MUTEX: *mut u32 = 0x4008_b0f8 as *mut u32;

#[no_mangle]
unsafe extern "C" fn main() -> ! {
    if cfg!(core = "0") {
        // enable the MAILBOX peripheral
        const SYSCON_AHBCLKCTRLSET0: *mut u32 = 0x4000_0220 as *mut u32;

        SYSCON_AHBCLKCTRLSET0.write_volatile(1 << 26);
    }

    let mut done = false;
    while !done {
        while MAILBOX_MUTEX.read_volatile() == 0 {
            // busy wait if the lock is held by the other core
        }
        atomic::compiler_fence(Ordering::Acquire);

        // we acquired the lock; now we have exclusive access to `SHARED`
        if SHARED >= 10 {
            // stop at some arbitrary point
            done = true;
        } else {
            SHARED += 1;
        }

        // release the lock & unblock the other core
        atomic::compiler_fence(Ordering::Release);
        MAILBOX_MUTEX.write_volatile(1);
    }

    match () {
        // only core #0 has a working ITM/SWO
        #[cfg(core = "0")]
        () => {
            use cortex_m::iprintln;

            if let Some(mut p) = cortex_m::Peripherals::take() {
                iprintln!(&mut p.ITM.stim[0], "DONE");
            }
        }

        #[cfg(not(core = "0"))]
        () => {}
    }

    loop {}
}
