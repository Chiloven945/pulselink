# pulselink fuzz targets

This directory is intentionally separate from the Cargo workspace and requires
a nightly toolchain with `cargo-fuzz`.

```bash
cargo install cargo-fuzz
cd pulselink-rs
cargo +nightly fuzz run relay_frame
cargo +nightly fuzz run b1_decode
```

Targets cover B0/BF encoding, B1/battery decoding, frequency conversion, V4 relay
frames, app messages, and device operations. Fuzzing is scheduled/manual; it is
not part of the stable CI test job.
