#![no_std]
#![no_main]

use embedded_hal::prelude::_embedded_hal_serial_Write;
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use cortex_m_rt::entry;
use stm32f4xx_hal::serial::{Serial, config};
use stm32f4xx_hal::gpio::{GpioExt};
use stm32f4xx_hal::rcc::RccExt;
use stm32f4xx_hal::time::Bps;

use embedded_hal::serial::Read;
use core::fmt::Write;   // (1)write!()マクロを使えるようにする
use stm32lib::uart::ErrorDetect;    // (2)追加するトレイトを使えるようにする

use stm32f4xx_hal::prelude::*;  // MHz()

use stm32f4xx_hal::pac::RCC;

const EXTERNAL_CLOCK: u16 = 8_u16;

#[entry]
fn main() -> ! {

    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();   // GPIOAのclockも有効にしてくれる （AHBENRレジスタ）
    let bps = Bps(115_200_u32); // (3)通信速度
    let seri_config = config::Config {  // (4)通信パラメーターの設定
        baudrate: bps,
        wordlength: config::WordLength::DataBits8,  // 実際には7ビット
        parity: config::Parity::ParityEven,
        stopbits: config::StopBits::STOP1,
        dma: config::DmaConfig::None,
    };

    let rcc = dp.RCC.constrain();

//    let clks = rcc.cfgr.freeze(); // 初期値でクロックを生成するコード

    let clks = rcc
        .cfgr
        .use_hse(8.MHz())   // 外部クロックを使う
        .bypass_hse_oscillator()    // 矩形波を使う（水晶振動子でなく発信器を使う）
        .sysclk(84.MHz())
        .pclk1(42.MHz())
        .pclk2(84.MHz())
        .freeze();

    let _hsebyp = unsafe { (*RCC::ptr()).cr.read().hsebyp().is_bypassed() };
    assert!(_hsebyp);

    assert!(clks.sysclk() == 84.MHz::<1_u32, 1_u32>());
    assert!(clks.pclk2() == 84.MHz::<1_u32, 1_u32>());
    assert!(clks.pclk1() == 42.MHz::<1_u32, 1_u32>());

    let mut serial = Serial::new(
        dp.USART2,
        (gpioa.pa2, gpioa.pa3),
        seri_config,
        &clks,
    ).unwrap(); // (5)Serial構造体の生成

    let sysclk = get_sysclk();    // 設定された M, N, P からクロックを計算してみる
    write!(serial, "sysclk={}MHz.\r\n", sysclk).unwrap();

    loop {
        while !serial.is_rx_not_empty() {}
        if serial.is_pe() {
            let _ = serial.read();  // 読み捨てる
            write!(serial, "\r\nParity error {}", "detected.\r\n").unwrap();
        }
        else if serial.is_fe() {
            let _ = serial.read();  // 読み捨てる
            write!(serial, "\r\nFraming error {}", "detected.\r\n").unwrap();
        }
        else if serial.is_ore() {
            let _ = serial.read();  // 読み捨てる
            write!(serial, "\r\nOver run error {}", "detected.\r\n").unwrap();
        }
        else if let Ok(c) = serial.read() {
            while !serial.is_tx_empty() {}
            serial.write(c).unwrap();
        }
    }
}

fn get_sysclk() -> u16 {

    let m = unsafe { (*RCC::ptr()).pllcfgr.read().pllm().bits() };
    let n = unsafe { (*RCC::ptr()).pllcfgr.read().plln().bits() };
    let p = unsafe { (*RCC::ptr()).pllcfgr.read().pllp().bits() };

    let cp = (p + 1) * 2;   // Pだけは変換が必要

    EXTERNAL_CLOCK * n / (m as u16 * cp as u16)    // 8M * N * 1/(M * P) = system clock

}

