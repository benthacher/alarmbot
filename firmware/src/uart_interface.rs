use core::usize;

use embassy_stm32::mode::Async;
use embassy_stm32::pac::{self, usart::Usart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use messages::{EncodableMessage, Message, FRAME_DELIMITER, MAX_ENCODED_LENGTH};

pub const RX_BUFFER_SIZE: usize = 1024;

pub struct UartInterface<'a> {
    uart_regs: pac::usart::Usart,
    rx_ringbuf: usart::RingBufferedUartRx<'a>,
    tx_uart: usart::UartTx<'a, Async>,
    message_handler: Option<fn(Message)>,
}

pub trait HalToPac {
    type PacPeripheral;
    fn pac(&self) -> Self::PacPeripheral;
}

macro_rules! hal_to_pac {
    ($periph:tt, $periph_name:tt) => {
        impl HalToPac for peripherals::$periph_name {
            type PacPeripheral = $periph;
            fn pac(&self) -> Self::PacPeripheral {
                pac::$periph_name
            }
        }
    };
}

hal_to_pac!(Usart, USART1);
hal_to_pac!(Usart, USART2);
hal_to_pac!(Usart, USART3);

bind_interrupts!(struct Irqs {
    USART3 => usart::InterruptHandler<peripherals::USART3>;
});

impl<'a> UartInterface<'a> {
    pub fn new<T: usart::Instance + HalToPac<PacPeripheral = Usart>>(
        uart: T,
        tx_pin: impl usart::TxPin<T>,
        rx_pin: impl usart::RxPin<T>,
        rx_dma: impl usart::RxDma<T>,
        tx_dma: impl usart::TxDma<T>,
        rx_buffer: &mut [u8; RX_BUFFER_SIZE],
    ) -> Self {
        let uart_regs = uart.pac();
        let (tx_uart, rx_uart) = usart::Uart::new(
            uart,
            rx_pin,
            tx_pin,
            Irqs,
            tx_dma,
            rx_dma,
            Default::default(),
        )
        .unwrap()
        .split();

        let rx_ringbuf = rx_uart.into_ring_buffered(rx_buffer);

        // set character match character to 0x00, the end of message character
        uart_regs.cr2().modify(|w| {
            w.set_add(FRAME_DELIMITER);
        });
        // enable character match interrupt
        uart_regs.cr1().modify(|w| {
            w.set_cmie(true);
        });

        Self {
            uart_regs,
            rx_ringbuf,
            tx_uart,
            message_handler: None,
        }
    }

    pub async fn send<'m>(&mut self, message: impl EncodableMessage) -> Result<(), usart::Error> {
        // This function uses the DMA completion interrupt to wake the Transfer so we're
        // free to use the UART interrupt for character matching
        let mut encoded_msg = [0u8; MAX_ENCODED_LENGTH];
        let encoded_length = message.encode(&mut encoded_msg);
        self.tx_uart.write(&encoded_msg[..encoded_length]).await
    }

    pub fn attach_message_handler(&mut self, handler: fn(Message)) {
        self.message_handler = Some(handler);
    }

    pub fn handle_interrupt(&mut self) {
        if self.uart_regs.isr().read().cmf() {
            self.uart_regs.icr().write(|w| w.set_cmf(true));

            self.character_match_waker
        }
    }

    pub async fn decode_task(&mut self) -> ! {
        loop {
            // await character match interrupt

            let mut tmp_buf = [0u8; MAX_ENCODED_LENGTH];
            let read_bytes = self.rx_ringbuf.read(&mut tmp_buf).await.unwrap();
            let message = Message::try_from(&mut tmp_buf[..read_bytes]).unwrap();

            match self.message_handler {
                Some(handler) => handler(message),
                None => (),
            }
        }
    }
}
