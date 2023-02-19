#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![allow(unused_imports)]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout
use rtic::app;

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use core::sync::atomic::{AtomicUsize, Ordering};
    // use cortex_m::peripheral::scb::VectActive::Interrupt;
    use stm32f4xx_hal::{
        gpio::{gpioa::PA5, gpioc::PC13, Alternate, Edge, Input, Output, Pin, PushPull},
        prelude::*,
        pac::Interrupt,
    };
    use dwt_systick_monotonic::{DwtSystick, ExtU32};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<'A', 5, Output<PushPull>>,
        button: Pin<'C', 13, Input>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<48_000_000>; // 48 MHz

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        // Cortex-M peripherals
        let mut _core : cortex_m::Peripherals = ctx.core;

        // Device specific peripherals
        let mut _device : stm32f4xx_hal::pac::Peripherals = ctx.device;

        // Set up the system clock.
        let rcc = _device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = _device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // Set up the button. On the Nucleo-F446RE it's connected to pin PC13.
        let gpioc = _device.GPIOC.split();
        let mut button = gpioc.pc13.into_floating_input();

        // Enable interrupts on the button
        let mut sys_cfg = _device.SYSCFG.constrain();
        button.make_interrupt_source(&mut sys_cfg);
        button.enable_interrupt(&mut _device.EXTI);
        button.trigger_on_edge(&mut _device.EXTI, Edge::Falling);

        // enable tracing and the cycle counter for the monotonic timer
        _core.DCB.enable_trace();
        _core.DWT.enable_cycle_counter();

        // Set up the monotonic timer
        let mono = DwtSystick::new(
            &mut _core.DCB,
            _core.DWT,
            _core.SYST,
            _clocks.hclk().to_Hz(),
        );

        blink::spawn().ok();

        (Shared {}, Local { button, led }, init::Monotonics(mono))
    }

    // The idle function is called when there is nothing else to do
    #[idle]
    fn idle(ctx: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(local = [led], priority = 4)]
    fn blink(ctx: blink::Context) {
        // reset the counter and get the old value.
        let count = COUNTER.swap(0, Ordering::SeqCst);
        defmt::info!("{}", count);
        ctx.local.led.toggle();
        blink::spawn_after(1.secs()).ok();
    }

    // This is the interrupt handler for the button, it is bound to the EXTI15_10 interrupt
    // as the the button is connected to pin PC13 and 13 is in the range 10-15.
    #[task(binds = EXTI15_10, local = [button])]
    fn on_exti(ctx: on_exti::Context) {
        // Clear the interrupt pending bit as rtic does not do this automatically.
        ctx.local.button.clear_interrupt_pending_bit();
        defmt::info!("incrementing");
        COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}
