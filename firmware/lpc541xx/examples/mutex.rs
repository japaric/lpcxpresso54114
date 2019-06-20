#![no_main]
#![no_std]

use core::sync::atomic::{self, Ordering};

use cortex_m::asm;
#[cfg(core = "0")]
use cortex_m::iprintln;
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
    // only core #0 has a functional ITM
    #[cfg(core = "0")]
    let mut itm = cortex_m::Peripherals::take().unwrap().ITM;

    let mut done = false;
    while !done {
        while MAILBOX_MUTEX.read_volatile() == 0 {
            // busy wait while the lock is held by the other core
        }
        atomic::fence(Ordering::Acquire);

        // we acquired the lock; now we have exclusive access to `SHARED`
        {
            let shared = &mut SHARED;

            if *shared >= 10 {
                // stop at some arbitrary point
                done = true;
            } else {
                *shared += 1;

                // log a message through the stimulus port #0
                #[cfg(core = "0")]
                iprintln!(&mut itm.stim[0], "[0] SHARED = {}", *shared);
            }
        }

        // release the lock & unblock the other core
        atomic::fence(Ordering::Release);
        MAILBOX_MUTEX.write_volatile(1);

        // artificial delay to let the *other* core take the mutex
        for _ in 0..1_000 {
            asm::nop();
        }
    }

    #[cfg(core = "0")]
    iprintln!(&mut itm.stim[0], "[0] DONE");

    loop {}
}
