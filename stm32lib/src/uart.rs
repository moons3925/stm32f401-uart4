use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::serial::Instance;
use stm32f4xx_hal::serial::Pins;

pub trait ErrorDetect {
    fn is_pe(&self) -> bool;    // パリティエラー
    fn is_fe(&self) -> bool;    // フレーミングエラー
    fn is_ore(&self) -> bool;   // オーバーランエラー
}

impl<USART, PINS> ErrorDetect for Serial<USART, PINS>
where
    PINS: Pins<USART>,
    USART: Instance,
{
    fn is_pe(&self) -> bool {
        unsafe { (*USART::ptr()).sr.read().pe().bit_is_set() }
    }
    fn is_fe(&self) -> bool {
        unsafe { (*USART::ptr()).sr.read().fe().bit_is_set() }
    }
    fn is_ore(&self) -> bool {
        unsafe { (*USART::ptr()).sr.read().ore().bit_is_set() }
    }
}

