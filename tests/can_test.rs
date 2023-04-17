#![no_std]
#![no_main]

use stm32f446_rtic as _; // global logger + panicking-behavior + memory layout

#[cfg(test)]
#[defmt_test::tests]
mod can_tests {
    use bxcan::filter::{self, Mask32};
    use bxcan::{Fifo, Frame, StandardId};
    use defmt::{assert, info};
    use stm32f446_rtic::can_shield::CanShield;
    use stm32f4xx_hal::{pac, prelude::*};

    #[test]
    // Requres connecting the CanShield to a bus with another CAN device running 
    // canshield_echo example (examples/canshield_echo.rs)
    fn test_shield() {
        let core = cortex_m::Peripherals::take().unwrap();
        let device = pac::Peripherals::take().unwrap();

        let rcc = device.RCC.constrain();
        let _clocks = rcc.cfgr.sysclk(180.MHz()).freeze();

        let gpioa = device.GPIOA.split();
        let gpiob = device.GPIOB.split();

        let shield = CanShield::new_rev1(
            gpioa.pa12,
            gpioa.pa11,
            gpiob.pb13,
            gpiob.pb5,
            device.CAN1,
            device.CAN2,
        )
        .unwrap();

        info!("CAN shield initialized");

        let mut can1 = shield.can1;
        let mut can2 = shield.can2;

        let mut test_frame1: [u8; 8] = [0; 8];
        test_frame1[0] = 'H' as u8;
        test_frame1[1] = 'e' as u8;
        test_frame1[2] = 'j' as u8;
        test_frame1[3] = 's' as u8;
        test_frame1[4] = 'a' as u8;
        test_frame1[5] = '!' as u8;
        test_frame1[6] = ' ' as u8;
        test_frame1[7] = '1' as u8;

        let mut test_frame2: [u8; 8] = [0; 8];
        test_frame2[0] = 'H' as u8;
        test_frame2[1] = 'e' as u8;
        test_frame2[2] = 'l' as u8;
        test_frame2[3] = 'l' as u8;
        test_frame2[4] = 'o' as u8;
        test_frame2[5] = '!' as u8;
        test_frame2[6] = ' ' as u8;
        test_frame2[7] = '2' as u8;

        let id_frame1 = StandardId::new(0x111).unwrap();
        let id_frame2 = StandardId::new(0x222).unwrap();
        let id_reply = StandardId::new(0x500).unwrap();

        let tx_frame1 = Frame::new_data(id_frame1, test_frame1);
        let tx_frame2 = Frame::new_data(id_frame2, test_frame2);

        info!("Sending frame 1: {:?}", tx_frame1);
        can1.transmit(&tx_frame1).unwrap();

        info!("Sending frame 2: {:?}", tx_frame2);
        can2.transmit(&tx_frame2).unwrap();

        loop {
            if let Ok(rx_frame1) = can1.receive() {
                info!("Received frame 1: {:?}", rx_frame1);
                assert_eq!(rx_frame1.id(), id_reply.into());
                assert_eq!(rx_frame1.data().unwrap().clone(), test_frame1.into());
                info!("CAN1 frame match successfull");
                break;
            }
        }

        loop {
            if let Ok(rx_frame2) = can2.receive() {
                info!("Received frame 2: {:?}", rx_frame2);
                assert_eq!(rx_frame2.id(), id_reply.into());
                assert_eq!(rx_frame2.data().unwrap().clone(), test_frame2.into());
                info!("CAN2 frame match successfull");
                break;
            }
        }
    }
}
