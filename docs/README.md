# Driver documentation

These documents define the durable VEML7700 driver contract.

## Reading order

1. [HARDWARE_CONTRACT.md](HARDWARE_CONTRACT.md) — interpreted device behavior
2. [INVARIANTS.md](INVARIANTS.md) — review-blocking truths and rejected patterns
3. [ARCHITECTURE.md](ARCHITECTURE.md) — ownership and dependency direction
4. [API_CONTRACT.md](API_CONTRACT.md) — public Rust surface
5. [TEST_PLAN.md](TEST_PLAN.md) — verification responsibilities and gaps

[DECISIONS.md](DECISIONS.md) records durable rationale,
[DOCUMENTATION_STANDARDS.md](DOCUMENTATION_STANDARDS.md) governs claims, and
[vendor/README.md](vendor/README.md) records source provenance without
redistributing vendor documents.

## Evidence boundary

Pure tests and strict scripted I²C establish codec and protocol behavior. The
independent model in `crates/veml7700-model` covers only `probe` and one
successful `measure_once` slice; see that crate README for the maintained claim.

The test-only fake sketches autonomous state, but its tests drive the fake
rather than the driver and it shares driver types and timing constants. It is
neither driver evidence nor independent cross-validation.

The repository contains no physical fixture, MCU application, hardware runner,
or evidence protocol. It makes no calibrated-optical, electrical, timing,
board, or silicon-qualification claim.
