# Decision log

## D-001 — Direct fixed-address I²C facade

`Veml7700<I2C>` owns the bus directly. The supported device has one fixed
address and no transport variation requiring an internal abstraction.

## D-002 — Preserve concrete bus errors

Errors retain the HAL error plus semantic operation/register/stage context.
Only address NACK is classified as absence during probe.

## D-003 — Strict low-byte-first codec

All 16-bit register transfers use little-endian wire order, protected by exact
transaction tests.

## D-004 — Bright-start is explicit policy

`MeasurementConfig::safe_bright_start()` is a value constructor, not hidden
driver state or automatic ranging.

## D-005 — Snapshot and fresh results are distinct

A snapshot may be retained or straddle channel refresh. Fresh capture controls
the domain and timing, freezes results, records provenance, and restores state.

## D-006 — Shutdown freezes the result pair

After conservative waiting, complete capture enters shutdown before sequential
ALS/white reads. This is a driver coherence policy, not a vendor atomicity claim.

## D-007 — Nominal integer scaling only

The crate provides integer micro-lux using the vendor table. Empirical and
system calibration remain outside the driver.

## D-008 — Threshold state is polled

The device has no dedicated interrupt pin. APIs use monitor/status language and
never own GPIO.

## D-009 — Monitor owns its complete domain

Gain, integration, thresholds, persistence, cadence, and active state are one
semantic domain protected against silent retargeting.

## D-010 — No undocumented clearing promise

The official sources do not establish reliable threshold-flag clearing
semantics, so the API promises observation only.

## D-011 — No VEML6030 abstraction

Family extraction waits for independently reviewed contracts for more than one
device.

## D-012 — Speculative physical infrastructure removed

Hardware runners, fixtures, plans, policies, transcripts, evidence structures,
and orchestration shims are not part of this driver product. Future physical
qualification begins from accepted driver and independent-model contracts.

## D-013 — Timing is bound to integration selection

Explicit timing may extend but never shorten the conservative wait and is
rejected before I²C if derived for a different integration time.

## D-014 — Fresh capture creates a known wake edge

Complete measurement installs the selected domain in shutdown before changing
to active and starting its wait.

## D-015 — Model independence is required

The independent cross-validation model implements I²C from the hardware contract
and must not reuse driver codecs, timing helpers, semantic types, or state
machines as its oracle. Autonomous cadence and threshold behavior belong there,
not in a driver-coupled fake. Injected ALS/white phase skew is test scheduling
topology rather than a silicon timing claim. The bounded slice and its nonclaims
are declared in
[`crates/veml7700-model/README.md`](../crates/veml7700-model/README.md).

## D-016 — Vendor documents are not redistributed

Track official URLs, revisions, retrieval facts, and available hashes without
committing vendor PDFs.

## D-017 — Local bounded validation

Private development uses one canonical local gate, `scripts/ci.sh`; only the
PowerShell launcher is retained beside it, because locating Git Bash on Windows
is real work and a POSIX passthrough wrapper is not. Hosted CI and generated
pack inventories are not required product surfaces *while the repository is
private*. That scope is deliberate: making the repository public activates the
contributor-support and hosted-CI expectations of the organization standards,
tracked in
[issue #11](https://github.com/photon-circus/ph-veml7700-als/issues/11).
The gate also tests the unpacked
distributable package, so packaging-only failures such as a stripped path
dependency cannot pass verification. Packaging is pinned to the repository
target directory because Cargo excludes only that path from workspace member
discovery; a configured target directory elsewhere in the repository would make
the extracted package untestable.

## D-018 — Release decisions remain explicit and independent

The driver uses the lifecycle-matching candidate version
`0.1.0-incubating.1` while retaining `publish = false`. Repository preparation,
visibility, and crates.io publication are separate maintainer-controlled
decisions. Model completeness, physical evidence, hardware qualification, and
`ph-hil` adoption limit their corresponding claims but do not gate an honestly
disclosed Incubating publication.

## D-019 — Model input limits are not device behavior

The model accepts `u8` addresses but declares a 7-bit I²C boundary. Values above
`0x7F` cannot exist on that bus, so they are reported as
`Unsupported::AddressOutOfRange` model limitations rather than fabricated device
NACKs. Other valid 7-bit addresses remain source-backed address NACKs. The model
never invents device behavior for inputs outside its declared domain.

## D-020 — Validating constructors own their fields

A public type whose constructor enforces a rule keeps its fields private, so the
rule cannot be bypassed by a struct literal.

`Thresholds::new` rejects `low > high` and returns `Option`. While its fields
were public, downstream code could build a reversed pair directly and pass it to
`arm_threshold_monitor`, which would program it. That produced an asymmetry the
driver should never have: it would write device state that `read_thresholds`
rejects as `ConfigurationError::ReversedThresholds` when read back. The fields
are now private with `low()` and `high()` accessors.

`ThresholdMonitorConfig` keeps public fields because it enforces no rule of its
own; it is a bundle whose only invariant now lives inside `Thresholds`.
