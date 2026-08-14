<!--
CONTRIBUTING.md holds the reviewability criteria. This template is the short
form. Delete sections that genuinely do not apply, and say why.
-->

## What changes and why

## Evidence source

Which of these produced the evidence for this change?

- [ ] Physical hardware
- [ ] Behavioral model (`ph-veml7700-als-model`)
- [ ] Scripted I²C or pure unit tests
- [ ] Documentation or tooling only; no behavior change

Mock, model, and simulated results must not be described as hardware evidence.
If this change touches registers, timing, reset, endianness, or bus behavior,
name the source that establishes the new behavior.

## Checklist

- [ ] Behavioral claims trace to `docs/HARDWARE_CONTRACT.md`.
- [ ] The public surface matches `docs/API_CONTRACT.md`, or that contract and
      `docs/DECISIONS.md` changed first.
- [ ] Every touched invariant has a protecting test.
- [ ] Exact I²C address, pointer, byte order, payload, and transaction count are
      asserted where transport behavior matters.
- [ ] The coupled fake was not turned into an oracle for the driver.
- [ ] `CHANGELOG.md` is updated beneath `Unreleased`.

## Local gate

The hosted workflow runs the `bounded` profile and reports its skipped checks.
It is not the release gate. Paste the final line of a full local run:

```text
[ci] PASS (full): N steps, 0 skipped.
```

## Claims this change does not make

<!--
Optional but encouraged. Naming what the change does *not* establish is how
this repository keeps evidence honest.
-->
