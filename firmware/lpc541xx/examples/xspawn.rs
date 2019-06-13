#![no_main]
#![no_std]

#[cfg(core = "0")]
use cortex_m::{iprintln, peripheral::ITM};
use panic_halt as _;

// heterogeneous dual core device: Cortex-M4F (#0) + Cortex-M0+ (#1)
#[rtfm::app(cores = 2, device = lpc541xx)]
const APP: () = {
    extern "C" {
        static mut ITM: ITM; // runtime initialized resource
    }

    #[init(core = 0, spawn = [ping])]
    fn init(mut c: init::Context) -> init::LateResources {
        iprintln!(&mut c.core.ITM.stim[0], "[0] init");

        c.spawn.ping(0).ok(); // cross-core message passing

        init::LateResources { ITM: c.core.ITM }
    }

    #[task(core = 0, resources = [ITM], spawn = [ping])]
    fn pong(c: pong::Context, x: u32) {
        iprintln!(&mut c.resources.ITM.stim[0], "[0] pong({})", x);

        c.spawn.ping(x + 1).ok(); // cross-core message passing
    }

    #[task(core = 1, spawn = [pong])]
    fn ping(c: ping::Context, x: u32) {
        // (the Cortex-M0+ core has no functional ITM to log messages)
        if x < 5 {
            c.spawn.pong(x + 1).ok(); // cross-core message passing
        }
    }

    extern "C" {
        #[core = 0]
        fn GINT0();

        #[core = 1]
        fn GINT0();
    }
};
