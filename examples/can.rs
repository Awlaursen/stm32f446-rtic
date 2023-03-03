#![no_main]
#![no_std]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {
    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    use core::sync::atomic::{AtomicUsize, Ordering};
    use stm32f4xx_hal::{
        can::Can,
        gpio::{self, gpioa::PA5, gpiob::{PB9, PB8}, Output, PushPull, Alternate},
        pac::CAN1,
        prelude::*,
    };
    use bxcan::filter::Mask32;
    use bxcan::{Fifo, Frame, StandardId};

    // Needed for scheduling monotonic tasks
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<48_000_000>; // 48 MHz

    // Holds the shared resources (used by multiple tasks)
    // Needed even if we don't use it
    #[shared]
    struct Shared {
        can: bxcan::Can<Can<CAN1, (PB9<Alternate<9>>, PB8<Alternate<9>>)>>
        // can: bxcan::Can<>
    }

    // Holds the local resources (used by a single task)
    // Needed even if we don't use it
    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
        test_frame: [u8; 8],
    }

    // Atomic counter
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    // The init function is called in the beginning of the program
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        // Cortex-M peripherals
        let mut _core : cortex_m::Peripherals = ctx.core;

        // Device specific peripherals
        let mut _device : stm32f4xx_hal::pac::Peripherals = ctx.device;

        // Set up the system clock.
        let rcc = _device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = _device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Initialize variables for can_send
        let mut test_frame: [u8;8] = [0;8];
        test_frame[1] = 1;
        test_frame[2] = 2;
        test_frame[3] = 3;
        test_frame[4] = 4;
        test_frame[5] = 5;
        test_frame[6] = 6;
        test_frame[7] = 7;

        // Set up CAN device 1.
        let gpiob = _device.GPIOB.split();
        let mut can1 = {
            let rx = gpiob.pb8.into_alternate::<9>();
            let tx = gpiob.pb9.into_alternate();
    
            // let can = Can::new(dp.CAN1, (tx, rx));
            // or
            let can = _device.CAN1.can((tx, rx));
    
            bxcan::Can::builder(can)
                // APB1 (PCLK1): 8MHz, Bit rate: 500kBit/s, Sample Point 87.5%
                // Value was calculated with http://www.bittiming.can-wiki.info/
                .set_bit_timing(0x001c_0000)
                .enable()
        };

            // Configure filters so that can frames can be received.
        let mut filters = can1.modify_filters();
        filters.enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

        // let _can2 = {
        //     let tx = gpiob.pb13.into_alternate();
        //     let rx = gpiob.pb12.into_alternate();

        //     let can = _device.CAN2.can((tx, rx));

        //     let can2 = bxcan::Can::builder(can)
        //         // APB1 (PCLK1): 8MHz, Bit rate: 500kBit/s, Sample Point 87.5%
        //         // Value was calculated with http://www.bittiming.can-wiki.info/
        //         .set_bit_timing(0x001c_0000)
        //         .enable();

        //     // A total of 28 filters are shared between the two CAN instances.
        //     // Split them equally between CAN1 and CAN2.
        //     filters.set_split(14);
        //     let mut slave_filters = filters.slave_filters();
        //     slave_filters.enable_bank(14, Fifo::Fifo0, Mask32::accept_all());
        //     can2
        // };

        // Drop filters to leave filter configuraiton mode.
        drop(filters);

        let can = can1;

        // enable tracing and the cycle counter for the monotonic timer
        _core.DCB.enable_trace();
        _core.DWT.enable_cycle_counter();

        // Set up the monotonic timer
        let mono = DwtSystick::new(
            &mut _core.DCB,
            _core.DWT,
            _core.SYST,
            clocks.hclk().to_Hz(),
        );

        defmt::info!("Init done!");
        blink::spawn_after(1.secs()).ok();
        can_send::spawn_after(1.secs()).ok();
        (Shared { can }, Local { led, test_frame }, init::Monotonics(mono))
    }

    // The idle function is called when there is nothing else to do
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    // The task functions are called by the scheduler
    #[task(local = [led])]
    fn blink(ctx: blink::Context) {
        ctx.local.led.toggle();
        defmt::info!("Blink!");
        blink::spawn_after(1.secs()).ok();
    }

    // send a meesage via CAN
    #[task(shared = [can], local = [test_frame],priority=1)]
    fn can_send(mut ctx: can_send::Context){
        let mut test_frame = ctx.local.test_frame;
        let id: u16 = 0x500;

        test_frame[1] = 1;
        test_frame[2] = 2;
        test_frame[3] = 3;
        test_frame[4] = 4;
        test_frame[5] = 5;
        test_frame[6] = 6;
        test_frame[7] = 7;

        test_frame[0] = COUNTER.fetch_add(1, Ordering::SeqCst) as u8;
        let frame = Frame::new_data(StandardId::new(id).unwrap(), *test_frame);
        
        defmt::info!("Sending frame with first byte: {}", test_frame[0]);

        ctx.shared.can.lock(|can| {
            can.transmit(&frame).unwrap()
        });

        can_send::spawn_after(1.secs()).ok();

    }
}
