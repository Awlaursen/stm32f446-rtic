#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use fugit as _;
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout // time abstractions

pub mod CanShield {
    use bxcan::filter::Mask32;
    use stm32f4xx_hal::{
        gpio::gpioa::{self, Parts},
        pac::{self, Peripherals},
        prelude::_stm32f4xx_hal_can_CanExt,
    };

    pub struct CanShield {
        can1: bxcan::Can<stm32f4xx_hal::pac::CAN1>,
        can2: bxcan::Can<stm32f4xx_hal::pac::CAN2>,
    }

    pub impl CanShield {
        // pub fn new(
        //     can1: bxcan::Can<stm32f4xx_hal::pac::CAN1>,
        //     can2: bxcan::Can<stm32f4xx_hal::pac::CAN2>,
        // ) -> Self {
        //     Self { can1, can2 }
        // }

        pub fn new_rev1(device: pac::Peripherals, gpioa: Parts, gpiob: Parts) -> CanShield {
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

            let mut filters =
                can1.modify_filters()
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
                If::FIFO0_MESSAGE_PENDING | If::FIFO0_FULL | If::FIFO0_OVERRUN
            });

            filters.set_split(14).slave_filters().enable_bank(
                14,
                bxcan::Fifo::Fifo1,
                Mask32::accept_all(),
            );

            // Drop filters to leave filter configuraiton mode.
            drop(filters);

            &mut Self { can1, can2 }
    
        }

        // pub fn can1(&mut self) -> &mut bxcan::Can<stm32f4xx_hal::pac::CAN1> {
        //     &mut self.can1
        // }

        // pub fn can2(&mut self) -> &mut bxcan::Can<stm32f4xx_hal::pac::CAN2> {
        //     &mut self.can2
        // }
    }
}
