#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use stm32l0xx_hal as hal;

use hal::{
    gpio::GpioExt,
    pac,
    rcc::{self, RccExt},
    syscfg::SYSCFG,
    usb::{UsbBus, USB},
};

use hex_display::HexDisplayExt;
use usb_device::prelude::*;
use usbd_mass_storage::USB_CLASS_MSC;
use usbd_scsi::{Scsi, BlockDevice};

struct MyBlockDevice {

}

impl BlockDevice for MyBlockDevice {
    const BLOCK_BYTES: usize = 64;

    fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), usbd_scsi::BlockDeviceError> {
        hprintln!("{}", block.hex());
        Ok(())
    }

    fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), usbd_scsi::BlockDeviceError> {
        Ok(())
    }

    fn max_lba(&self) -> u32 {
        1
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(rcc::Config::hsi16());
    let mut syscfg = SYSCFG::new(dp.SYSCFG, &mut rcc);
    let hsi48 = rcc.enable_hsi48(&mut syscfg, dp.CRS);

    let gpioa = dp.GPIOA.split(&mut rcc);

    let usb = USB::new(dp.USB, gpioa.pa11, gpioa.pa12, hsi48);
    let usb_bus = UsbBus::new(usb);

    // let mut msc = MscClass::new(
    //     &usb_bus,
    //     64u16,
    //     usbd_mass_storage::InterfaceSubclass::ScsiTransparentCommandSet,
    //     usbd_mass_storage::InterfaceProtocol::BulkOnlyTransport,
    // );
    let bd = MyBlockDevice{};
    let rev: [u8; 1] = [1];
    let mut scsi = Scsi::new(&usb_bus, 64u16, bd, "fake_vendor", "fake_product", rev);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0930, 0x6545))
        .manufacturer("Fake company")
        .product("Fake MSC")
        .serial_number("TEST")
        .device_class(USB_CLASS_MSC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut scsi]) {
            continue;
        }

        // let mut buf = [0u8; 64];
        // let mut countstr = [0u8; 8];

        // match msc.read_packet(&mut buf) {
        //     Ok(count) if count > 0 => {
        //         rled.set_high().ok();
        //         // Echo back in upper case
        //         hprintln!("Read packet {} bytes", count.numtoa_str(10, &mut countstr));
        //         hprintln!("{}", buf[0..count].hex());

        //         let mut write_offset = 0;
        //         while write_offset < count {
        //             hprintln!("Write packet {} bytes", count.numtoa_str(10, &mut countstr));
        //             hprintln!("{}", buf[0..count].hex());
        //             match msc.write_packet(&buf[write_offset..count]) {
        //                 Ok(len) if len > 0 => {
        //                     write_offset += len;
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         rled.set_low().ok();
        //     }
        //     _ => {}
        // }
    }
}
