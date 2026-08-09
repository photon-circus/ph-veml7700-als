# Contributing

Changes begin with the contracts, not the implementation.

1. Update the relevant hardware, architecture, API, invariant, test, or decision
   document.
2. Explain observable behavior and failure semantics.
3. Add strict scripted-I²C tests and, where time or autonomous behavior matters,
   behavioral-model tests.
4. Run `python tools/validate-pack.py` and `tools/check.sh` or
   `tools/check.ps1`.
5. Keep mock HIL results distinct from physical evidence.

Do not add a public raw-register API, hidden configuration cache, automatic
optical correction, or driver-owned retry policy without a reviewed contract
change.
