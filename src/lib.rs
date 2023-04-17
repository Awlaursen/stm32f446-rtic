#![no_std]
#![no_main]

use defmt_rtt as _; // global logger
use fugit as _;
use panic_probe as _; // panic handler
use stm32f4xx_hal as _; // memory layout // time abstractions

pub mod can_shield {
    use bxcan::{filter::Mask32, Fifo};
    use defmt::info;
    use stm32f4xx_hal::{
        can::Can,
        gpio::{Alternate, PA11, PA12, PB13, PB5},
        pac::{CAN1, CAN2},
        prelude::_stm32f4xx_hal_can_CanExt,
    };

    pub struct CanShield {
        pub can1: bxcan::Can<Can<CAN1, (PA12<Alternate<9>>, PA11<Alternate<9>>)>>,
        pub can2: bxcan::Can<Can<CAN2, (PB13<Alternate<9>>, PB5<Alternate<9>>)>>,
    }

    impl CanShield {
        pub fn new_rev1(
            pa12: PA12,
            pa11: PA11,
            pb13: PB13,
            pb5:  PB5,
            can1 : CAN1,
            can2 : CAN2,
        ) -> Result<Self, ()> {
            let mut can1 = {
                let rx = pa11.into_alternate::<9>();
                let tx = pa12.into_alternate::<9>();

                let can = can1.can((tx, rx));

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

            let mut binding = can1.modify_filters();
            let filters = binding.enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

            let mut can2 = {
                let rx = pb5.into_alternate::<9>();
                let tx = pb13.into_alternate::<9>();

                let can = can2.can((tx, rx));

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

            filters.set_split(14).slave_filters().enable_bank(
                14,
                bxcan::Fifo::Fifo1,
                Mask32::accept_all(),
            );

            // Drop filters and binding to move the CAN instances into the `CanShield` struct.
            drop(filters);
            drop(binding);

            Ok(Self { can1, can2 })
        }

        // pub fn can1(&mut self) -> &mut bxcan::Can<stm32f4xx_hal::pac::CAN1> {
        //     &mut self.can1
        // }

        // pub fn can2(&mut self) -> &mut bxcan::Can<stm32f4xx_hal::pac::CAN2> {
        //     &mut self.can2
        // }
    }
}
