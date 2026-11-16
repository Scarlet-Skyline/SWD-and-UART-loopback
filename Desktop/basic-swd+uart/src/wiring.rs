//! Complete the two TODOs in this file.

use embassy_stm32::{
    Peripherals,
    can::Can,
    gpio::{Flex, Level, Output, Speed},
    usart::{BufferedUart, Config},
};
use static_cell::StaticCell;

use crate::swd::Swd;

pub struct Ports {
    pub can: Can<'static>,
    pub uart: Option<BufferedUart<'static>>,
}

pub fn configure(p: Peripherals) -> (crate::board::Usb, Ports, Option<Swd>) {
    let usb = crate::board::usb(p.USB_OTG_HS, p.PA12, p.PA11);

    // TODO 1: Pick two exposed 3.3 V GPIOs and pass them to Swd::new.
    // Use Output for SWCLK (initially low), and Flex for bidirectional SWDIO.
    let swclk = Output::new(p.PA0, Level::Low, Speed::VeryHigh);
    let mut swdio = Flex::new(p.PA2); //PWM所对应的引脚?
    swdio.set_as_output(Speed::VeryHigh);
    let swd = Some(Swd::new(swclk, swdio));
    //let swd = None;

    // TODO 2: Configure a buffered UART at 115200 baud, 8N1.
    // Identify the peripheral, RX/TX pins and interrupt from the schematic.
    // Bind the interrupt and provide static RX and TX buffers.
    let mut uart_config = Config::default();
    uart_config.baudrate = 115200;
    // 8N1 已经是 default 配置，无需额外修改

    static TX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();

    let tx_buf = TX_BUF.init([0u8; 256]);
    let rx_buf = RX_BUF.init([0u8; 256]);

    // 注意：这里的 Irqs 需要你在工程中通过 bind_interrupts! 宏定义
    // 例如：bind_interrupts!(struct Irqs { UART8 => usart::InterruptHandler<peripherals::UART8>; });
    // 这里假设你已经定义了该结构体，名为 Irqs
    let uart = BufferedUart::new(
        p.USART1,
        p.PA10, // RX
        p.PA9,
        //crate::board::UartIrqs, // <--- 修复：使用 board.rs 中定义的中断实例          // TX
        //crate::Irqs,    // 修复：传入具体的中断绑定实例，而不是类型名
        &mut tx_buf[..], // 显式转换为切片
        &mut rx_buf[..], // 显式转换为切片
        //crate::Irqs,
        crate::board::UartIrqs, // 修复：传入具体的中断绑定实例，而不是类型名         // RX buffer
        uart_config,
    )
    .unwrap();
    // 修复：既然配置了，就要用 Some 包起来，不要被后面的 None 覆盖
    let uart = Some(uart);
    let can = crate::board::can1(p.FDCAN1, p.PD0, p.PD1);

    (usb, Ports { can, uart }, swd)
}