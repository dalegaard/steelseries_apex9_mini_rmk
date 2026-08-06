# SteelSeries™ Apex 9 Mini RMK firmware

This is a custom firmware for the Apex 9 Mini optical switch keyboard by
SteelSeries™. The firmware is written in Rust, using the RMK framework.

Currently only ISO layout is implemented, as I only have this board available. I
believe the same PCB is used for multiple different layouts, so the same
scanning engine should work for all of them.

This firmware was made public in the hopes that it will be useful to someone
some day. The author is not in any way affiliated with SteelSeries, and they
have not endorsed this in any way.

Currently, only keyboard scanning is implemented. No attempt has yet been made
to play around with the LEDs.

## Compilation

Simply:

```sh
cargo build --release
```

This requires `flip-link`.

## Flashing

The keyboard can be flashed via the 5-pin header underneath the space bar. The
pinout is printed next to the header. The stock firmware is flash protected, and
unlocking the protection erases the firmware. See [recovery](#recovery) for some
notes on how to revert to the stock firmware.

The 5-pin header can be programmed with a J-Link, STLink, or any other probe
compatible with a STM32L412RB MCU.

Once the board has been cleared, the firmware can be flashed either with DFU
over USB, or via the already connected debugger. The compiled firmware image
lives at `target/thumbsv7em-none-eabihf/release/steelseries_apex9_mini_rmk`. It
can be flashed e.g. with `probe-rs`:

```sh
probe-rs download --chip stm32l412rb target/thumbv7em-none-eabihf/release/steelseries_apex9_mini_rmk
```

To flash with DFU, first create a raw binary and then flash it with `dfu-util`:

```sh
arm-none-eabi-objcopy target/thumbv7em-none-eabihf/release/steelseries_apex9_mini_rmk -O binary /tmp/apex9.bin
dfu-util -d 04d8:df11 -a 0 -D /tmp/apex9.bin -s 0x08000000:leave
```

Once the target has been flashed, DFU mode can be entered by holding down the
top-left key(escape) while plugging in the keyboard.

## Development

For developing on the firmware, the easiest is to simply use
`cargo run --release` with an attached SWD debugger. It will program and launch
the firmware on the board, and output log messages.

## Background

The following sections contain some background on the keyboard from a hardware
perspective.

### Hardware

The keyboard is in many ways almost the same as the Apex Pro Mini, which is
supported and well documented in
[Zephyr](https://docs.zephyrproject.org/latest/boards/steelseries/apex_pro_mini/doc/index.html).

The main difference appears to be in how the keys are scanned. The Apex 9 Mini
uses optical keys instead of electrical contacts, and therefore doesn't use a
standard matrix. Each key has a small LED and a phototransistor. When the key is
in the up state, the phototransistor has an unobstructed view of the LED. When
the key is pressed down, the view of the LED is blocked and the phototransistor
output signal changes. Blocking the path partially, i.e. by pressing the key
down less than all the way, gives a partial blockage. This is utilized to
provide the different actuation levels in the stock firmware(gaming and typing
modes).

### Recovery

It is not possible to read out the stock firmware before unlocking the flash,
but the stock firmware can nevertheless be restored. The SteelSeries™ Engine
software package for Windows contains the firmware loaded on the device, at the
time of writing the file is called `firmware-apex-9-mini-1.19.9.bin`. The
firmware image must be loaded at address `0x0800_9000`. Since the bootloader has
been cleared by the chip erase, some method of jumping to the firmware image
must be provided. A script, `make_apex9_image.py` is provided in this
repository, which will create a binary that can be loaded at `0x0800_0000`
instead.
