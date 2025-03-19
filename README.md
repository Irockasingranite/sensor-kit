# Sensor Kit Demo

This application demonstrates the use of a variety of peripherals found on the Arduino Sensor Kit.
The button connected to pin D4 cycles through a variety of modes, some of which can be controlled by
the potentiometer dial connected to pin A0.

The firmware is compatible with STM32 Nucleo-F413ZH and Raspberry Pi Pico2 Boards. The default
target is the Nucleo-F413ZH, see instructions below for building and flashing for Pico2 boards.

## Wiring for Pico2

The RP Pico2 does not come with an Arduino header, so to use it with the sensor kit, some additional
wiring is necessary.

Connect pins according to this table:
| Pico2  | Sensor Kit |
| ------ | ---------- |
| GND    | GND        |
| VBUS   | 3V3        |
| GP0    | SDA        |
| GP1    | SCL        |
| GP2    | D4         |
| GP3    | D5         |
| GP4    | D6         |
| GP26   | A0         |
| GP27   | A2         |
| GP28   | A3         |


## Building and Flashing

To build this project, make sure `probe-rs` is installed:
```
cargo binstall probe-rs-tools
```

If `cargo-binstall` is missing, install that first:
```
cargo install cargo-binstall
```

### Nucleo-F413ZH

The Nucleo-F413ZH is configured as the default target. As such, you can simply run
```
cargo build --release
```

to build the firmware, and

```
cargo flash --release --chip=STM32F413ZHTx
```

to flash it.

To run the firmware with `probe-rs` attached for capturing log output, run
```
cargo run --release
```

### Pico2

To build for Pico2 targets, you'll have invoke a few extra options:
```
cargo build --release --no-default-features -F rp-pico --target thumbv8m.main-none-eabihf
```

This does the following:
- `--no-default-features`: Disables the feature `nucleo-f413zh`, which is otherwise active by
  default.
- `-F rp-pico`: Enables the `rp-pico` feature instead.
- `--target thumbv8m.main-none-eabihf`: Selects the target triple, overriding the configures default
  triple `thumbv7em-none-eabihf` used for the Nucleo board.

To flash to the Pico2, use `probe-rs` as follows:
```
probe-rs download --chip RP235x target/thumbv8m.main-none-eabihf/release/sensor-kit
```

Not that this command does not invoke `cargo build` first, so make sure to rebuild after making
changes.

Similarly,
```
probe-rs attach --chip RP235x target/thumbv8m.main-none-eabihf/release/sensor-kit
```
can be used to capture log output from a running board.
