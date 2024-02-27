# picow-rust-iot-demonstrator

# Overview

This is a concise example showcasing the utilization of the Raspberry Pi Pico W (=PicoW) as an IoT sensor device in Rust. It serves as a visibility study highlighting the feasibility of employing the PicoW as an IoT device with Rust as the programming language. Additionally, it leverages Rust's async functionality.

![IoT Rust Demonstrator Architecture](./picow-iot.png "IoT Rust Demonstrator Architecture")

The setup consists of the components: 

 * PicoW Sensor (`picow-temperature-sensor`)

   As sample data the temperature is measure periodically.   
   The value is forwarded to the IoT Broker via TCP.
 * Sensor Simulator (`sensor-simulator`)

   If no PicoW is available you can also use a sensor simulator.
 * IoT Data Bridge (`iot-data-bridge`)

   Acts as a bridge for data, receiving temperature readings from the sensor and forwarding them to a sqlite database.
 * IoT WebServer (`iot-webserver`)

   serves the web services and web page.
 * Sqlite DB

   As datastorage a filebased sqlite database is used accessed from rust with `iot-db-accessor`.
   
 * IoT Explorer (`iot-explorer`)

   A CLI application that lists the sensor data and can also create testdata.
 * IoT WebPage (`iot-webserver`)

   serves a dashboard of the sensor data.  
   
When new sensor data becomes available in the database, the `iot-explorer` and "IoT WebPage" automatically update whenever they are open.

# Building and Executing the IoT Demonstrator

**⚠️ WARNING:** Minimum supported rust version: 1.74. To update: `rustup toolchain update stable`

Building is separated into two steps:
Pico part and the host part (everything else).
So if you have no PicoW you can test it with the `sensor-simulator`.

If you use vscode with for the described steps are corresponding tasks configured (in [[.vscode/tasks.json]]). This can be executed with Ctrl+Shift+B ("Run Tasks").

**⚠️ WARNING:** Execute the `cargo` commands from the project root. Otherwise, the database will not be found, as it is configured via a relative path.

## Install sqlx-cli

```bash
cargo install sqlx-cli
```

## Build host part

```bash
cargo build
```

## Start iot-web-server

Start in one bash terminal the iot-webserver (or with vscode excute task "01-iot-webserver"):

```bash
cargo run --bin iot-webserver
```

After starting the iot-webserver, you see link to the dashboard page, e.g.: http://localhost:3000

## Start IoT Explorer

You can also view the latest data via cli ((or with vscode excute task "02-iot-explorer")):

```bash
cargo run --bin iot-explorer last --follow
```

With `cargo run --bin iot-explorer` you see the help page.

With `cargo run --bin iot-explorer help last` you see the help page for the last command.

## Start iot-data-bridge

Start in one bash terminal the iot-data-bridge (or with vscode excute task "03-iot-data-bridge"):

```bash
cargo run --bin iot-data-bridge
```

## Start Sensor Data Producer (PicoW or Simulator)

⚠️ **Attention:** Data acquisition must use only one source: a) `sensor-simulator`, OR b) `~picow-temperature-sensor`

### a) `sensor-simulator`

```bash
cargo run --release --bin sensor-simulator
```

### b) `picow-temperature-sensor`

This project is not part of the cargo workspace.

#### Setup the Rust Environment for PicoW

Please ensure that you have setup the rust development environment for the PicoW and test it, e.g. blinky example from the embassy project (details [here](./picow-temperature-sensor/README.md))

### Edit `.env` file

⚠️ **WARNING:** Before you can use the PicoW project, please ensure that you **edit the `.env` file and complete the TODOs**.

### Build and flash the PicoW

```bash
cd picow-temperature-sensor
cargo run --release
```

After flashing the pico tries to periodically send data to the `iot-data-bridge`. To stop this disconnect the power cable.

