#![no_main]
#![no_std]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {
    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    use stm32f4xx_hal::{
        can::Can,
        gpio,
        gpio::{gpioa::PA5, Output, PushPull, Alternate},
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
        can: bxcan::Can<Can<stm32f4xx_hal::pac::CAN1, (gpio::Pin<'B', 9, Alternate<9>>, gpio::Pin<'B', 8, Alternate<9>>)>>
        // can: bxcan::Can<>
    }

    // Holds the local resources (used by a single task)
    // Needed even if we don't use it
    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,

    }

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

        let _can2 = {
            let tx = gpiob.pb13.into_alternate();
            let rx = gpiob.pb12.into_alternate();

            let can = _device.CAN2.can((tx, rx));

            let can2 = bxcan::Can::builder(can)
                // APB1 (PCLK1): 8MHz, Bit rate: 500kBit/s, Sample Point 87.5%
                // Value was calculated with http://www.bittiming.can-wiki.info/
                .set_bit_timing(0x001c_0000)
                .enable();

            // A total of 28 filters are shared between the two CAN instances.
            // Split them equally between CAN1 and CAN2.
            filters.set_split(14);
            let mut slave_filters = filters.slave_filters();
            slave_filters.enable_bank(14, Fifo::Fifo0, Mask32::accept_all());
            can2
        };

        // Drop filters to leave filter configuraiton mode.
        drop(filters);

        let mut can = can1;

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
        (Shared { can }, Local { led }, init::Monotonics(mono))
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
    #[task(shared = [can])]
    fn can_send(mut ctx: can_send::Context){
        let mut test: [u8; 8] = [0; 8];
        let mut count: u8 = 0;
        let id: u16 = 0x500;

        test[1] = 1;
        test[2] = 2;
        test[3] = 3;
        test[4] = 4;
        test[5] = 5;
        test[6] = 6;
        test[7] = 7;

        test[0] = count;
        let test_frame = Frame::new_data(StandardId::new(id).unwrap(), test);
        // block!(can.transmit(&test_frame)).unwrap();

        ctx.shared.can.lock(|can| {
            can.transmit(&test_frame).unwrap()
        });

        if count < 255 {
            count += 1;
        } else {
            count = 0;
        }

    }
}
