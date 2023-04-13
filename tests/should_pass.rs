#![no_std]
#![no_main]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

#[cfg(test)]
#[defmt_test::tests]
mod should_pass {
    use defmt::{
        info,
        assert
    };

    #[test]
    fn should_pass() {
        info!("Hello, world!");
        assert!(true)
    }

}