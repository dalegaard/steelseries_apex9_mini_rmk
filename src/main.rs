#![no_main]
#![no_std]

#[macro_use]
mod macros;
mod dfu;
mod keymap;
mod matrix;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::{DMA1_CH1, USB};
use embassy_stm32::usb::{self, Driver};
use embassy_stm32::{Config, bind_interrupts, dma};
use embassy_stm32::{adc::AdcChannel, gpio::Output};
use panic_probe as _;
use rmk::config::{BehaviorConfig, PositionalConfig, RmkConfig};
use rmk::keyboard::Keyboard;
use rmk::processor::builtin::wpm::WpmProcessor;
use rmk::usb::UsbTransport;
use rmk::{KeymapData, initialize_keymap, run_all};

use crate::matrix::{ApexOpticalMatrix, Thresholds};

bind_interrupts!(struct Irqs {
    USB => usb::InterruptHandler<USB>;
    DMA1_CHANNEL1 => dma::InterruptHandler<DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    dfu::check_enter();

    info!("RMK start!");
    // RCC config
    let mut config = Config::default();
    config.rcc.pll = Some(embassy_stm32::rcc::Pll {
        source: embassy_stm32::rcc::PllSource::HSI,

        prediv: embassy_stm32::rcc::PllPreDiv::DIV1,
        mul: embassy_stm32::rcc::PllMul::MUL10,

        divp: None,
        divq: None,
        divr: Some(embassy_stm32::rcc::PllRDiv::DIV2),
    });
    config.rcc.hsi = true;
    config.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_R;

    // Initialize peripherals
    let p = embassy_stm32::init(config);

    // Pin config
    let (col_inputs, row_outputs) = config_optical_matrix_pins_stm32!(
        peripherals: p,
        input: [PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PC4, PC5, PC0, PC1, PC2, PC3],
        output: [PB0, PB1, PB2, PB10, PB11]
    );

    // Usb driver
    let driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    // Keyboard config
    let rmk_config = RmkConfig::default();

    // Initialize the matrix
    let thresholds = Thresholds {
        thres_trigger: 0x80.into(),
        thres_untrigger: 0x70.into(),
    };
    let mut matrix = ApexOpticalMatrix::new(
        p.ADC1,
        p.DMA1_CH1,
        Irqs,
        col_inputs,
        row_outputs,
        &thresholds,
    );

    if matrix.check_key(0, 0).await {
        info!("Bootmagic held, entering bootloader");
        dfu::enter();
    }

    // Initialize the keymap
    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let mut behavior_config = BehaviorConfig::default();
    let per_key_config = PositionalConfig::default();
    let keymap = initialize_keymap(&mut keymap_data, &mut behavior_config, &per_key_config).await;

    let mut keyboard = Keyboard::new(&keymap);

    let mut usb_transport = UsbTransport::new(driver, rmk_config.device_config);
    let mut wpm_processor = WpmProcessor::new();

    // Start
    run_all!(matrix, usb_transport, wpm_processor, keyboard).await;
}
