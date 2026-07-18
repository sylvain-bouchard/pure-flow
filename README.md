# Pure Flow

A production-grade, asynchronous Rust firmware implementation for smart environmental monitoring and ventilation control. This project leverages the Embassy framework on the Nordic nRF52840 architecture to drive multi-sensor data fusion and intelligent air-exchange efficiency.

## Key Features

*   **Asynchronous Architecture:** Built on top of the `embassy-executor` for ultra-low-power, deterministic, non-blocking multitasking.
*   **Sensirion Sensor Suite:** Native I2C driver integration for high-precision environmental tracking:
    *   **SFA30:** Formaldehyde, temperature, and relative humidity.
    *   *Roadmap:* Multi-sensor fusion with **SCD40** (CO2) and **SEN55** (Particulates/VOCs).
*   **PID-Based Fan Control:** Active closed-loop feedback control to dynamically adjust ventilation speed based on real-time air quality metrics.
*   **Optimized Power States:** Utilizes the nRF52840's deep sleep modes between active sensor sampling intervals to maximize battery/solar runtime.

## Hardware Stack

*   **MCU:** Seeed Studio XIAO nRF52840 (ARM Cortex-M4F running at 64 MHz).
*   **Communication:** I2C bus configuration for sensor arrays; future expansion via Sub-GHz/LoRa for decentralized, whole-home mesh networking.

## Toolchain & Setup

This project uses standard bare-metal Rust tooling. Ensure you have the `thumbv7em-none-eabihf` target installed.

```bash
# Add the target
rustup target add thumbv7em-none-eabihf

# Install flashing/debugging tools (probe-rs recommended)
cargo install probe-rs --features cli

# Flash the board
cargo run --release
