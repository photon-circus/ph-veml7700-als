# Contributing

Changes begin with the contracts, not the implementation.

1. Update the relevant hardware, architecture, API, invariant, test, or decision
   document.
2. Explain observable behavior and failure semantics.
3. Add strict scripted-I²C tests and, where time or autonomous behavior matters,
   behavioral-model tests.
4. Run the complete local CI runner: `tools/check.sh` or `tools/check.ps1`.
5. Keep mock HIL results distinct from physical evidence.

Do not add a public raw-register API, hidden configuration cache, automatic
optical correction, or driver-owned retry policy without a reviewed contract
change.

Do not add GitHub Actions workflow YAML while the crate is under development.
The pack validator enforces local-only CI orchestration.
