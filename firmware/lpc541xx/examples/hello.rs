#![deny(warnings)]
#![no_main]
#![no_std]

use core::sync::atomic::{self, Ordering};

use lpc541xx as _;
use panic_halt as _;

#[no_mangle]
unsafe extern "C" fn main() -> ! {
    match () {
        // only core #0 has a working ITM/SWO
        #[cfg(core = "0")]
        () => {
            use cortex_m::iprintln;

            // NOTE the ITM is initialized by pyOCD (see `pyocd_user.py`)
            if let Some(mut p) = cortex_m::Peripherals::take() {
                iprintln!(&mut p.ITM.stim[0], "Hello, world!");
            }
        }

        #[cfg(not(core = "0"))]
        () => {}
    }

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}
