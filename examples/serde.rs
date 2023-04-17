// Serde example with RTIC on the STM32F446RE Nucleo board
// This example shows how to use the serde crate to serialize and deserialize data

#![no_main]
#![no_std]

use rtic::app;
use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

mod app {
    use defmt::info;
    use dwt_systick_monotonic::{DwtSystick, ExtU32};
    use serde::{Deserialize, Serialize};
    use stm32f4xx_hal::{
        gpio::{gpioa::PA5, Output, PushPull},
        prelude::*,
    };

    // Struct to hold the data we want to serialize
    #[derive(Serialize, Deserialize)]
    struct Data {
        a: u32,
        b: u32,
    }

    // Needed for scheduling monotonic tasks
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = DwtSystick<45_000_000>; // 48 MHz

    // Holds the shared resources (used by multiple tasks)
    // Needed even if we don't use it
    #[shared]
    struct Shared {
        data: Data,
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
        let mut _core: cortex_m::Peripherals = ctx.core;

        // Device specific peripherals
        let mut _device: stm32f4xx_hal::pac::Peripherals = ctx.device;

        // Set up the system clock.
        let rcc = _device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(45.MHz()).freeze();

        // Set up the LED. On the Nucleo-F446RE it's connected to pin PA5.
        let gpioa = _device.GPIOA.split();
        let led = gpioa.pa5.into_push_pull_output();

        // enable tracing and the cycle counter for the monotonic timer
        _core.DCB.enable_trace();
        _core.DWT.enable_cycle_counter();

        // Set up the monotonic timer
        let mono = DwtSystick::new(&mut _core.DCB, _core.DWT, _core.SYST, clocks.hclk().to_Hz());

        // Create the data we want to serialize
        let data = Data { a: 1, b: 2 };

        // Serialize the data
        let serialized = serde::Serialize::serialize(&data).unwrap();

        info!("Serialized: {:?}", serialized);

        // Return the initialized resources
        (Shared { data }, Local { led }, init::Monotonics(mono))
    }

    // The idle function is called when no other task is ready to run
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        defmt::info!("idle");
        loop {
            cortex_m::asm::wfi();
        }
    }

    // The task function is called periodically
    #[task(local = [led])]
    fn blink(ctx: blink::Context) {
        defmt::info!("task");

        // Toggle the LED
        ctx.local.led.toggle();
    }
}
