macro_rules! config_optical_matrix_pins_stm32 {
    (peripherals: $p:ident, input: [$($in_pin:ident), *], output: [$($out_pin:ident), +]) => {
        {
            let output_pins = [
                $(Output::new($p.$out_pin, embassy_stm32::gpio::Level::High, embassy_stm32::gpio::Speed::VeryHigh)),+
            ];
            let input_pins = [
                $($p.$in_pin.degrade_adc()),+
            ];
            (input_pins, output_pins)
        }
    };
}
