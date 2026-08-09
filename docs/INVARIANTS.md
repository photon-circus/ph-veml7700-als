# Review-blocking invariants

## Runtime

- **I-1:** runtime code is `no_std`, allocation-free, and unsafe-free.
- **I-2:** `Veml7700<I2C>` stores only `i2c: I2C`.
- **I-3:** construction performs no I/O; release returns the exact bus.
- **I-4:** every word transfer is low byte then high byte.
- **I-5:** the address is fixed at `0x10`; callers cannot supply another address.

## Measurement truth

- **I-6:** snapshot reads never claim freshness.
- **I-7:** a fresh result includes measurement configuration and applied wait.
- **I-7a:** fresh capture installs its domain in shutdown, then creates an explicit
  shutdown-to-active wake edge before timing begins.
- **I-7b:** explicit timing is conservative-or-longer and must be derived for the
  same integration-time selection as the requested measurement.
- **I-8:** ALS/white coherence is explicitly qualified.
- **I-9:** nominal lux carries units and never claims calibration.
- **I-10:** white counts are not silently converted with ALS scaling.
- **I-11:** reserved configuration, PSM, and threshold-status encodings are errors.

## Autonomous state

- **I-12:** no hardware state is cached.
- **I-13:** shutdown retention is documented on every snapshot path.
- **I-14:** complete measurement restoration failures preserve the captured sample
  and declare device state uncertain.
- **I-15:** cleanup failure before capture reports both primary and recovery errors.

## Threshold monitor

- **I-16:** no interrupt GPIO abstraction exists.
- **I-17:** the monitored domain includes gain, integration time, thresholds,
  persistence, power-saving cadence, and active state.
- **I-18:** ordinary methods cannot silently retarget an enabled monitor.
- **I-19:** arm sequence is disable-first and enable-last.
- **I-20:** status reads promise observation only, not undocumented clearing.

## HIL and evidence

- **I-21:** the publishable crate has no `ph-hil` dependency.
- **I-22:** mock evidence is void for physical capability claims.
- **I-23:** optical accuracy requires a calibrated reference, retained identity and
  calibration state, characterized geometry/window/diffuser, and source metadata.
- **I-24:** source review, compiler success, or a logic capture alone is not an
  optical accuracy claim.

## Distribution safety

- **I-25:** Cargo publication remains hard-disabled. Repository automation may
  build a package for inspection, but it owns no registry credential and cannot
  publish the crate.

## Rejected shortcuts

| Shortcut | Defect |
| --- | --- |
| `from_be_bytes` / MSB-first writes | wrong wire order |
| `read_lux()` with no provenance | stale and calibration ambiguity |
| driver-owned auto-ranging | hidden policy and unpredictable latency |
| treating ALS_INT_EN as a pin | hardware has no dedicated pin |
| changing PSM while monitor active | silently changes qualification time |
| applying high-lux polynomial globally | application-dependent correction |
| preserving unknown reserved bits as normal | accepts undocumented state |
| retrying a failed restore invisibly | hides state uncertainty |
