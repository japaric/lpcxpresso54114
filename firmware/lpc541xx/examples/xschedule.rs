//! ping pong message passing using the `schedule` API

#![no_main]
#![no_std]

#[cfg(core = "0")]
use cortex_m::{iprintln, peripheral::ITM};
use lpc541xx::Duration;
#[cfg(core = "0")]
use lpc541xx::Instant;
use panic_halt as _;

const PERIOD: u32 = 6_000_000; // CPU clock cycles or about half a second

#[rtfm::app(cores = 2, device = lpc541xx, monotonic = lpc541xx::CTIMER0)]
const APP: () = {
    extern "C" {
        static mut ITM: ITM;
    }

    #[init(core = 0, schedule = [ping])]
    fn init(mut c: init::Context) -> init::LateResources {
        iprintln!(&mut c.core.ITM.stim[0], "[0] init");

        // run this task in half a second from now
        let _ = c.schedule.ping(c.start + Duration::from_cycles(PERIOD), 0);

        init::LateResources { ITM: c.core.ITM }
    }

    #[task(core = 0, resources = [ITM], schedule = [ping])]
    fn pong(c: pong::Context, x: u32) {
        let now = Instant::now();
        let scheduled = c.scheduled;

        iprintln!(
            &mut c.resources.ITM.stim[0],
            "[0] pong({}) scheduled @ {:?} ran @ {:?}",
            x,
            scheduled,
            now
        );

        let _ = c
            .schedule
            .ping(scheduled + Duration::from_cycles(PERIOD), x + 1);
    }

    #[task(core = 1, schedule = [pong])]
    fn ping(c: ping::Context, x: u32) {
        if x < 5 {
            let _ = c
                .schedule
                .pong(c.scheduled + Duration::from_cycles(PERIOD), x + 1);
        }
    }

    extern "C" {
        #[core = 0]
        fn GINT0();

        #[core = 0]
        fn GINT1();

        #[core = 1]
        fn GINT0();

        #[core = 1]
        fn GINT1();
    }
};
