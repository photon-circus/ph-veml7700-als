# Contributing

Read `AGENTS.md` and the normative documents under `docs/` before changing
behavior.

A contribution is reviewable only when:

1. Behavioral claims trace to `docs/HARDWARE_CONTRACT.md`.
2. The public surface matches `docs/API_CONTRACT.md`, or that contract and the
   decision log change first.
3. Every touched invariant has a protecting test.
4. Exact I²C address, pointer, byte order, payload, and transaction count are
   asserted where transport behavior matters.
5. Autonomous behavior is tested without converting the coupled fake into an
   oracle for the driver. Independent-model tests use the model crate and public
   driver APIs, not driver codecs.
6. The canonical local gate passes.

Run `./tools/check.sh`, or `./tools/check.ps1` from PowerShell.

Do not add raw-register APIs, cached state, automatic optical correction,
speculative physical infrastructure, or remove `publish = false`.
