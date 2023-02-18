#![no_std]


use core::sync::atomic::{AtomicUsize, Ordering};

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout
use fugit as _; // time abstractions
