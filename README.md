# Sensor Kit Demo

This application demonstrates the use of a variety of peripherals found on the Arduino Sensor Kit.
The button connected to pin D4 cycles through a variety of modes, some of which can be controlled by
the potentiometer dial connected to pin A0.

The firmware is compatible with STM32 Nucleo-F413ZH and Raspberry Bi Pico2 Boards. The default
target is the Nucleo-F413ZH, see instructions below for building and flashing for Pico2 boards.

To build this project, make sure `probe-rs` is installed:
```
cargo binstall probe-rs-tools
```

If `cargo-binstall` is missing, install that first:
```
cargo install cargo-binstall
```

Then, build the project using cargo:
```
cargo build --release
```

Flash it via
```
cargo flash --release --chip=STM32F413ZHTx
```

Or run it directly via
```
cargo run --release
```
to capture log output.

## Building for Pico2

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
