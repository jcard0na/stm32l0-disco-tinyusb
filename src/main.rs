#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

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
use usbd_dfu::{DFUClass, DFUManifestationError, DFUMemError, DFUMemIO};
use usbd_mass_storage::USB_CLASS_MSC;

struct MyMem {
    buffer: [u8; 64],
    flash_memory: [u8; 1024],
}

impl DFUMemIO for MyMem {
    const MEM_INFO_STRING: &'static str = "@Flash/0x00000000/1*1Kg";
    const INITIAL_ADDRESS_POINTER: u32 = 0x0;
    const PROGRAM_TIME_MS: u32 = 8;
    const ERASE_TIME_MS: u32 = 50;
    const FULL_ERASE_TIME_MS: u32 = 50;
    const TRANSFER_SIZE: u16 = 64;

    fn read(&mut self, address: u32, length: usize) -> Result<&[u8], DFUMemError> {
        // TODO: check address value
        let offset = address as usize;
        hprintln!("read");
        Ok(&self.flash_memory[offset..offset + length])
    }

    fn erase(&mut self, _address: u32) -> Result<(), DFUMemError> {
        // TODO: check address value
        self.flash_memory.fill(0xff);
        // TODO: verify that block is erased successfully
        Ok(())
    }

    fn erase_all(&mut self) -> Result<(), DFUMemError> {
        // There is only one block, erase it.
        self.erase(0)
    }

    fn store_write_buffer(&mut self, src: &[u8]) -> Result<(), ()> {
        self.buffer[..src.len()].copy_from_slice(src);
        Ok(())
    }

    fn program(&mut self, address: u32, length: usize) -> Result<(), DFUMemError> {
        // TODO: check address value
        let offset = address as usize;

        // Write buffer to a memory
        self.flash_memory[offset..offset + length].copy_from_slice(&self.buffer[..length]);

        // TODO: verify that memory is programmed correctly
        Ok(())
    }

    fn manifestation(&mut self) -> Result<(), DFUManifestationError> {
        // Nothing to do to activate FW
        hprintln!("manifestation");
        Ok(())
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

    let my_mem = MyMem {
        buffer: [0u8; 64],
        flash_memory: [0u8; 1024],
    };

    let mut dfu = DFUClass::new(&usb_bus, my_mem);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xf055, 0xdf11))
        .manufacturer("Fake company")
        .product("Fake MSC")
        .serial_number("TEST")
        // .device_class(USB_CLASS_MSC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut dfu]) {
            continue;
        }
    }
}
