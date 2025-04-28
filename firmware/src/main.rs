#![no_main]
#![no_std]

use embassy_stm32::rcc;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// mod usb_interface_test;

mod uart_interface;

#[rtic::app(device = embassy_stm32::pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use messages::Message;

    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        uart_interface: uart_interface::UartInterface<'static>,
    }

    #[init]
    fn init(_c: init::Context) -> (Shared, Local) {
        rtt_init_print!();

        let mut config = embassy_stm32::Config::default();
        config.rcc.hsi = true;

        // 16 MHz HSI -> div 4, mul 85, div 2 -> 170 MHz sysclk
        config.rcc.pll = Some(rcc::Pll {
            source: rcc::PllSource::HSI,
            prediv: rcc::PllPreDiv::DIV4,
            mul: rcc::PllMul::MUL85,
            divp: None,
            divq: None,
            divr: Some(rcc::PllRDiv::DIV2),
        });
        config.rcc.sys = rcc::Sysclk::PLL1_R;
        config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;
        config.rcc.apb1_pre = rcc::APBPrescaler::DIV1;
        config.rcc.apb2_pre = rcc::APBPrescaler::DIV1;
        config.rcc.boost = true;

        let p = embassy_stm32::init(config);

        let mut UART_RX_BUFFER: [u8; 1024] = [0u8; 1024];

        let uart = uart_interface::UartInterface::new(
            p.USART3,
            p.PB10,
            p.PB11,
            p.DMA1_CH1,
            p.DMA1_CH2,
            &mut UART_RX_BUFFER,
        );

        rprintln!("init finished");

        (
            Shared {},
            Local {
                uart_interface: uart,
            },
        )
    }

    #[task(priority = 0)]
    async fn uart_tx(_c: uart_tx::Context) {
        loop {
            rprintln!("uart tx running!");
        }
    }
}
