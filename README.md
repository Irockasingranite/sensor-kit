# Rusty STM

This is a template for getting started with developing rust based firmware for STM32
microcontrollers using embassy.

Defaults are filled for an STM32 Nucleo-F413ZH board. For other boards, adjust as necessary.

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
