#![no_main]
#![no_std]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1, USART2])]
mod app {
    use bxcan::{Data, Frame, StandardId};
    use core::{
        str,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use defmt::*;
    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    use stm32f446_rtic::can_shield::CanShield;
    use stm32f4xx_hal::{
        can::Can,
        gpio::{
            gpioa::{PA11, PA12, PA5},
            gpiob::{PB13, PB5},
            Alternate, Output, PushPull,
        },
        pac::CAN1,
        pac::CAN2,
        prelude::*,
    };

    // Needed for scheduling monotonic tasks
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<180_000_000>; // 180 MHz

    // Holds the shared resources (used by multiple tasks)
    // Needed even if we don't use it
    #[shared]
    struct Shared {
        can1: bxcan::Can<Can<CAN1, (PA12<Alternate<9>>, PA11<Alternate<9>>)>>,
        can2: bxcan::Can<Can<CAN2, (PB13<Alternate<9>>, PB5<Alternate<9>>)>>,
    }

    // Holds the local resources (used by a single task)
    // Needed even if we don't use it
    #[local]
    struct Local {
        led: PA5<Output<PushPull>>,
    }

    // Atomic counter
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    // The init function is called in the beginning of the program
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        info!("init");

        // Cortex-M peripherals
        let mut _core: cortex_m::Peripherals = ctx.core;

        // Device specific peripherals
        let mut _device: stm32f4xx_hal::pac::Peripherals = ctx.device;

        // Set up the system clock.
        let rcc = _device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(180.MHz()).freeze(); 

        debug!("AHB1 clock: {} Hz", clocks.hclk().to_Hz());
        debug!("APB1 clock: {} Hz", clocks.pclk1().to_Hz());

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = _device.GPIOA.split();
        let gpiob = _device.GPIOB.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Set up CAN device 1
        let shield = CanShield::new_rev1(
            gpioa.pa12,
            gpioa.pa11,
            gpiob.pb13,
            gpiob.pb5,
            _device.CAN1,
            _device.CAN2,
        )
        .unwrap();

        let mut can1 = shield.can1;
        let mut can2 = shield.can2;

        // enable tracing and the cycle counter for the monotonic timer
        _core.DCB.enable_trace();
        _core.DWT.enable_cycle_counter();

        // Set up the monotonic timer
        let mono = DwtSystick::new(&mut _core.DCB, _core.DWT, _core.SYST, clocks.hclk().to_Hz());

        info!("Init done!");
        blink::spawn_after(1.secs()).ok();
        (Shared { can1, can2 }, Local { led }, init::Monotonics(mono))
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
        debug!("Blink!");
        blink::spawn_after(1.secs()).ok();
    }

    // send a meesage via CAN
    #[task(shared = [can1, can2], priority=2)]
    fn can_send(mut ctx: can_send::Context, _ch: u8, _data: Data) {
        let id: u16 = 0x500;

        let frame = Frame::new_data(StandardId::new(id).unwrap(), _data);

        info!(
            "Sending frame: {}",
            str::from_utf8(&frame.clone().data().unwrap()).unwrap_or("Invalid UTF-8")
        );

        // Send the frame
        match _ch {
            1 => {
                ctx.shared.can1.lock(|can1| can1.transmit(&frame).unwrap());
            }
            2 => {
                ctx.shared.can2.lock(|can2| can2.transmit(&frame).unwrap());
            }
            _ => {
                error!("Invalid channel");
            }
        }
    }

    // receive a message via CAN1
    #[task(binds = CAN1_RX0, shared = [can1])]
    fn can1_receive(ctx: can1_receive::Context) {
        let mut can1 = ctx.shared.can1;
        let frame = can1.lock(|can1| can1.receive().unwrap());

        let data = frame.data().unwrap();

        info!(
            "Received frame: {}",
            str::from_utf8(data).unwrap_or("Invalid UTF-8")
        );

        can_send::spawn(1, data.clone()).ok();
    }

    // receive a message via CAN2
    // Note: CAN2_RX1 is used instead of CAN2_RX0 because CAN2 is set up to use FIFO 1 in the CanShield implementation
    #[task(binds = CAN2_RX1, shared = [can2])]
    fn can2_receive(ctx: can2_receive::Context) {
        let mut can2 = ctx.shared.can2;
        let frame = can2.lock(|can2| can2.receive().unwrap());

        let data = frame.data().unwrap();

        info!(
            "Received frame: {}",
            str::from_utf8(data).unwrap_or("Invalid UTF-8")
        );

        can_send::spawn(2, data.clone()).ok();
    }
}
