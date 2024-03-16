# Raspberry Pico W Temperatur Sensor

This is a simple IoT demonstrator. It reads the onboard temperature sensor and sends the measured value via TCP to a server (`iot-data-brige`).

# Raspberry Pi Pico W Rust Embedded Development Setup Guide

## Introduction

This guide will walk you through setting up your Raspberry Pi Pico W for Rust embedded development. 

There are two ways to set up your environment:
 * `debug-probe`: Pico connected to a Debug Probe (referred as `debug-probe`). For debugging the application, this is mandatory. Due to the ease of development option, `debug-probe` is preferred way ([details](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html)).
 * `UF2`: Pico connected directly to the host computer. ⚠️ **Attention: No debugging is possible. Before flashing the pico it has to be in BOOTSEL-MODE: Hold down the BOOTSEL button when you plug in your Pico. ([details](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html#resetting-flash-memory))**.

Please follow this guide according your option.

### Import Links and Documents:

[Raspberry Pi Pico](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html)

[Getting Started with Pico pdf](https://datasheets.raspberrypi.com/pico/getting-started-with-pico.pdf)

[Raspberry Pi Debug Probe](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html)

## Prerequisites

Before you begin, ensure you have the following installed on your development machine:
- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
- [Rust Embedded Tools for Raspberry Pi Pico](https://github.com/rp-rs/rp-hal?tab=readme-ov-file#getting-started)

## 1. Install Prerequisites

Install Rust, Cargo, and Git using the provided links.

## 2. Set Up the Rust Embedded Toolchain

Open a terminal and install the `thumbv6m-none-eabi` target for cross-compilation to ARM Cortex-M0+:

```bash
rustup target add thumbv6m-none-eabi
```

## 3. Set Up `embassy`

Clone the repo as described [here](https://embassy.dev/book/dev/getting_started.html). 

Run the `blinky` example in `embassy/examples/rp` by executing `cargo run --bin blinky --release`. 

**Note:** The onboard LED on the PicoW is not connected to GPIO 25 as on the Pico without W. So you either have to connect an LED or accept, that you cannot see the output...

**Note:** Depending on your setup, you have to adapt the example's `.cargo/config.toml` as described in the next section for our project here.

## 4. Configure `.cargo/config.toml`

Edit the file [`.cargo/config.toml`](.cargo/config.toml) so that only one runner is configured. Comment the other.

### a) debug-probe (default)

```
runner = "probe-rs run --chip RP2040"
```

### b) UF2

```
runner = "elf2uf2-rs --deploy --serial --verbose"
```

## 5. Build and flash the PicoW

see [`../README.md`](../README.md)
