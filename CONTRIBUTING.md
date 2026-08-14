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
5. Autonomous behavior is tested in the independent model. Conformance tests
   use the model crate and public driver APIs, not driver codecs.
6. The canonical local gate passes.

Run `./scripts/ci.sh`, or `./tools/check.ps1` from PowerShell.

Do not add raw-register APIs, cached state, automatic optical correction, or
speculative physical infrastructure. Do not change repository visibility or
registry publication state without the corresponding recorded maintainer
decision and release review.
