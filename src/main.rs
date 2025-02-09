#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embedded_hal::{delay::DelayNs, i2c::I2c};
use panic_probe as _;
use rp2040_hal as hal;

use hal::pac;

use fugit::RateExtU32;  // ファイルの先頭に追加
use core::fmt::Write;  // Write トレイトをアンダースコアでインポート
use heapless::String;


#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rp2040_hal::entry]
fn main() -> ! {
    info!("Program start!");
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut timer = rp2040_hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    let sda_pin = pins.gpio16.into_pull_up_input().into_function::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_pull_up_input().into_function::<hal::gpio::FunctionI2C>();
    
    // I2C0 の初期化（400kHz）
    let mut i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400_000u32.Hz(),
        &mut pac.RESETS,
        &clocks.system_clock,
    );

    loop {
        timer.delay_ms(2000);

        info!("\nI2C Bus Scan");
        info!("     0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F");

        let mut buf: String<256> = String::new();
        let mut count = 0;
    
        for addr in 0..=127u8 {
    
            let mut data = [0u8; 1];
            let result = i2c.read(addr, &mut data);
            
            match result {
                Ok(_) => {
                    let _ = core::write!(&mut buf, " {:02X} ", addr);
                },
                Err(_) => {
                    let _ = core::write!(&mut buf, "  . ");
                },
            }

            if addr % 16 == 15 {
                info!("{}0 {=str}", count, buf.as_str());
                buf = String::new();
                count += 1;
            }
        }

        info!("Scan End.")
    }
}
