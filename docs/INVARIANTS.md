# Invariants and rejections

## Runtime

- **I-1:** runtime code is `no_std`, allocation-free, and unsafe-free.
- **I-2:** `Veml7700<I2C>` stores only the I²C resource.
- **I-3:** construction performs no I/O; release returns the exact bus.
- **I-4:** every word transfer is low byte then high byte.
- **I-5:** the address is fixed at `0x10`.

## Measurement truth

- **I-6:** snapshots never claim freshness.
- **I-7:** fresh results include configuration and applied timing provenance.
- **I-8:** fresh capture installs its domain in shutdown and creates a known
  wake edge before timing.
- **I-9:** timing cannot be shorter than the conservative bound or belong to a
  different integration selection.
- **I-10:** ALS/white coherence is qualified explicitly.
- **I-11:** nominal lux never claims calibration; white counts are not converted
  using ALS scaling.
- **I-12:** reserved encodings are errors.

## Autonomous state and recovery

- **I-13:** no hardware state is cached.
- **I-14:** shutdown retention is documented on snapshot paths.
- **I-15:** restoration failures preserve captured samples and state uncertainty.
- **I-16:** cleanup failures report both primary and recovery errors.

## Threshold monitor

- **I-17:** no interrupt GPIO abstraction exists.
- **I-18:** the domain includes gain, integration, thresholds, persistence,
  cadence, and active state.
- **I-19:** ordinary methods cannot silently retarget an enabled monitor.
- **I-20:** arming is disable-first and enable-last.
- **I-21:** status reads promise observation only.

## Model and distribution

- **I-22:** the coupled fake is not described as independent driver validation.
- **I-23:** an independent mock must use the I²C boundary and derive behavior
  from the hardware contract without driver codecs as its oracle. The first
  slice is declared in `crates/veml7700-model/README.md`.
- **I-24:** host-model results are not physical or calibrated-optical evidence.
- **I-25:** vendor PDFs remain untracked.
- **I-26:** Cargo publication remains disabled.

## Rejected shortcuts

| Shortcut | Defect |
| --- | --- |
| MSB-first word transfer | wrong wire order |
| `read_lux()` without provenance | stale/calibration ambiguity |
| driver-owned auto-ranging | hidden policy and latency |
| treating monitor enable as a pin | device has no dedicated interrupt pin |
| changing cadence while monitoring | changes qualification time silently |
| global high-lux polynomial | application-dependent correction |
| accepting reserved bits | invents undocumented state |
| hiding restoration failure | erases state uncertainty |
| reusing driver codecs in the independent mock | tests implementation against itself |
