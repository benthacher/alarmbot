#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use embassy_stm32::rcc;

#[rtic::app(device = embassy_stm32::pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(mut c: init::Context) -> (Shared, Local) {
        rtt_init_print!();

        let mut config = embassy_stm32::Config::default();
        config.rcc.hsi = true;

        // 25 MHz oscillator -> div 5, mul 192, div 2 -> 480 MHz sysclk
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

        rprintln!("init finished");

        (
            Shared {},
            Local {},
        )
    }

    #[task(priority = 0)]
    async fn uart_tx(_c: uart_tx::Context) {
        loop {
            rprintln!("uart tx running!");
        }
    }
}