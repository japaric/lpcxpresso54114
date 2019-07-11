#![no_main]
#![no_std]

use cortex_m::{iprintln, peripheral::ITM};
use panic_halt as _;

// heterogeneous dual core device: Cortex-M4F (#0) + Cortex-M0+ (#1)
#[rtfm::app(cores = 2, device = lpc541xx)]
const APP: () = {
    struct Resources {
        itm: ITM,
    }

    #[init(core = 0, spawn = [ping])]
    fn init(mut c: init::Context) -> init::LateResources {
        iprintln!(&mut c.core.ITM.stim[0], "[0] init");

        // cross core message passing
        let _ = c.spawn.ping(0);

        init::LateResources { itm: c.core.ITM }
    }

    #[task(core = 0, resources = [itm], spawn = [ping])]
    fn pong(c: pong::Context, x: u32) {
        iprintln!(&mut c.resources.itm.stim[0], "[0] pong({})", x);

        // cross core message passing
        let _ = c.spawn.ping(x + 1);
    }

    #[task(core = 1, spawn = [pong])]
    fn ping(c: ping::Context, x: u32) {
        // (the Cortex-M0+ core has no functional ITM to log messages)

        if x < 5 {
            // cross core message passing
            let _ = c.spawn.pong(x + 1);
        }
    }

    extern "C" {
        #[core = 0]
        fn GINT0();

        #[core = 1]
        fn GINT0();
    }
};
