#![no_std]
#![no_main]

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
// use cortex_m_semihosting::hprintln;
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32l0xx_hal as hal;

use hal::{
    delay::Delay,
    gpio::GpioExt,
    pac,
    prelude::ToggleableOutputPin,
    rcc::{self, RccExt},
};

#[entry]
fn main() -> ! {
    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(rcc::Config::hsi16());
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let mut rled = gpioa.pa5.into_push_pull_output();
    let mut gled = gpiob.pb4.into_push_pull_output();
    let mut delay = Delay::new(cp.SYST, rcc.clocks);

    rled.toggle().ok();
    loop {
        rled.toggle().ok();
        gled.toggle().ok();
        delay.delay_ms(1000u32);
        // hprintln!("boing");
    }
}
