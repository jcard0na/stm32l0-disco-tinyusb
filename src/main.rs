#![no_std]
#![no_main]

// use cortex_m_semihosting::hprintln;
// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use stm32l0xx_hal as hal;

use hal::{
    gpio::GpioExt,
    pac,
    prelude::OutputPin,
    rcc::{self, RccExt},
    syscfg::SYSCFG,
    usb::{UsbBus, USB},
};

use usb_device::prelude::*;
use usbd_mass_storage::{MscClass, USB_CLASS_MSC};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(rcc::Config::hsi16());
    let mut syscfg = SYSCFG::new(dp.SYSCFG, &mut rcc);
    let hsi48 = rcc.enable_hsi48(&mut syscfg, dp.CRS);

    let gpioa = dp.GPIOA.split(&mut rcc);

    let mut rled = gpioa.pa5.into_push_pull_output();
    let usb = USB::new(dp.USB, gpioa.pa11, gpioa.pa12, hsi48);
    let usb_bus = UsbBus::new(usb);

    let mut msc = MscClass::new(
        &usb_bus,
        64u16,
        usbd_mass_storage::InterfaceSubclass::Ufi,
        usbd_mass_storage::InterfaceProtocol::BulkOnlyTransport,
    );
    // let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0930, 0x6545))
        .manufacturer("Fake company")
        .product("Fake MSC")
        .serial_number("TEST")
        .device_class(USB_CLASS_MSC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut msc]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match msc.read_packet(&mut buf) {
            Ok(count) if count > 0 => {
                rled.set_high().ok();
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match msc.write_packet(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
                rled.set_low().ok();
            }
            _ => {}
        }
    }
}
