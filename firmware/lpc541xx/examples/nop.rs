#![no_main]
#![no_std]

use core::sync::atomic::{self, Ordering};

use lpc541xx as _;
use panic_halt as _;

#[inline(never)]
#[no_mangle]
unsafe extern "C" fn main() -> ! {
    loop {
        atomic::fence(Ordering::SeqCst);
    }
}
