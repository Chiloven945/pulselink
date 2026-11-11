# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- M0: virtual Cargo workspace with eight library crates and an integration-test package.
- M0: workspace lints, Rust 2024 edition, `resolver = "3"`, Apache-2.0 license, CI baseline, docs
  and spec skeletons.
- M1: `pulselink-core` domain types: device identity, channels, intensity, observed state,
  capabilities, events, and errors.
- M1: `pulselink-pattern` waveform timeline types: pulse samples, 100 ms stereo frames, timeline,
  pattern, and playback cursor.
- M2: `pulselink-protocol` Coyote V3 BLE codecs: GATT identifiers, B0/BF encoding, B1/battery
  decoding, and pulse interval conversion.
- M2: `pulselink-protocol` DG-LAB V4 WebSocket types: relay frames, RPC envelopes, device model,
  events, and device operations.
- M2: cross-language test vectors for Coyote V3 and V4.
- M3: `pulselink-transport` device link SPI with `GattLink`, `Central`, `Connector`, `Backend`,
  `ScanFilter`, and a controllable mock backend.
- M4: `pulselink-ble` btleplug backend: adapter selection, filtered/deduplicated scanning with
  cancellation, connection setup, write/read/notify, reconnect, error mapping, and a hardware probe.
- M5: `pulselink-ws` V4 transport: socket lifecycle, hello/target id, multi-app registry, RPC
  manager, device cache, `device.op` submission/completion, heartbeat/ping, close handling, and a
  mock relay.
