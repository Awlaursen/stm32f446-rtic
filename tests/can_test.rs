#![no_std]
#![no_main]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

#[cfg(test)]
#[defmt_test::tests]
mod can_tests {
    use stm32f446_rtic::CanShield::CanShield;
    use bxcan::filter::{self, Mask32};
    use bxcan::{Fifo, Frame, StandardId};
    use cortex_m_rt::entry;
    use defmt::{assert, info};
    use nb::block;
    use stm32f4xx_hal::{pac, prelude::*};

    #[test]
    fn can1_call_response() {
        let core = cortex_m::Peripherals::take().unwrap();
        let device = pac::Peripherals::take().unwrap();

        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(180.MHz()).freeze();

        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();

        let mut can1 = {
            let rx = gpioa.pa11.into_alternate::<9>();
            let tx = gpioa.pa12.into_alternate::<9>();

            let can = device.CAN1.can((tx, rx));

            info!("CAN1, waiting for 11 recessive bits...");
            bxcan::Can::builder(can)
                // APB1 (PCLK1): 45MHz, Bit rate: 1MBit/s, Sample Point 87.5%
                // Value was calculated with http://www.bittiming.can-wiki.info/
                .set_bit_timing(0x001b0002)
                .set_automatic_retransmit(true)
                .enable()
        };

        can1.enable_interrupts({
            use bxcan::Interrupts as If;
            If::FIFO0_MESSAGE_PENDING | If::FIFO0_FULL | If::FIFO0_OVERRUN
        });

        let mut filters = can1
            .modify_filters()
            .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

        let mut can2 = {
            let rx = gpiob.pb5.into_alternate::<9>();
            let tx = gpiob.pb13.into_alternate::<9>();

            let can = device.CAN2.can((tx, rx));

            info!("CAN2, waiting for 11 recessive bits...");
            bxcan::Can::builder(can)
                // APB1 (PCLK1): 45MHz, Bit rate: 1MBit/s, Sample Point 87.5%
                // Value was calculated with http://www.bittiming.can-wiki.info/
                .set_bit_timing(0x001b0002)
                .set_automatic_retransmit(true)
                .enable()
        };

        can2.enable_interrupts({
            use bxcan::Interrupts as If;
            If::FIFO1_MESSAGE_PENDING | If::FIFO1_FULL | If::FIFO1_OVERRUN
        });

        // Split the filters evenly between CAN1 and CAN2
        filters
            .set_split(14)
            .slave_filters()
            .enable_bank(14, Fifo::Fifo1, Mask32::accept_all());

        // Drop filters to leave filter configuraiton mode.
        drop(filters);

    }

    #[test]
    fn test_shield() {
        let core = cortex_m::Peripherals::take().unwrap();
        let device = pac::Peripherals::take().unwrap();

        let rcc = device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(180.MHz()).freeze();

        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();

        let shield = CanShield::new_rev1(device, gpioa, gpiob);
    }
}
