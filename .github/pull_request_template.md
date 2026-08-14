<!--
CONTRIBUTING.md holds the full reviewability criteria. This template is the
short form. Delete sections that genuinely do not apply, and say why.
-->

## Purpose and value

<!-- What problem does this solve, for whom, and why is it worth carrying? -->

## Scope and governing decision

- Governing issue, proposal, or release record:
- Responsibility or invariant strengthened:
- Explicitly outside this PR:

<!-- Keep the PR independently acceptable or rejectable. -->

## Evidence source

Which of these produced the evidence for this change?

- [ ] Physical hardware
- [ ] Behavioral model (`ph-veml7700-als-model`)
- [ ] Scripted I²C transport tests
- [ ] Pure unit tests
- [ ] Documentation or tooling only; no behavior change

Mock, model, and simulated results must not be described as hardware evidence.
If this change touches registers, timing, reset, endianness, or bus behavior,
name the source that establishes the new behavior.

## Contract and compatibility

<!--
Observable behavior changes, API and versioning impact, supported targets and
features, and any new failure, timing, memory, cancellation, or restoration
semantics. Write "No contract change" when that is true.
-->

## Evidence

| Claim or gate | Command, artifact, or observation | Result |
| --- | --- | --- |
|  |  |  |

<!--
Name the exact commit and environment when a result was not produced by this
PR's checks. Distinguish hardware, model, scripted transport, and inference.
A skipped check is not a passed check.
-->

## Local gate

The full local gate is the release authority; hosted CI runs a bounded subset
and is contributor feedback only. Paste the final line of a full run:

```text
[ci] PASS (full): N steps, 0 skipped.
```

## Checklist

- [ ] Behavioral claims trace to `docs/HARDWARE_CONTRACT.md`.
- [ ] The public surface matches `docs/API_CONTRACT.md`, or that contract and
      `docs/DECISIONS.md` changed first.
- [ ] Every touched invariant has a protecting test.
- [ ] Exact I²C address, pointer, byte order, payload, and transaction count are
      asserted where transport behavior matters.
- [ ] Autonomous behavior is tested in the independent model. Conformance tests
      use the model crate and public driver APIs, not driver codecs.
- [ ] README, contract, or support documentation was updated, or this PR explains
      why no update is needed.
- [ ] `CHANGELOG.md` is updated beneath `Unreleased`, or the change is
      demonstrably internal.
- [ ] License, upstream provenance, and package contents remain correct; no
      vendor document became tracked.
- [ ] Known limitations and rejected approaches are recorded where future work
      could otherwise repeat them.

## Handoff and remaining work

- Commit verified:
- Toolchain and tool versions:
- Remaining risks or unresolved decisions:
- Follow-up issues:

## Claims this change does not make

<!--
Optional but encouraged. Naming what the change does *not* establish is how this
repository keeps evidence honest.
-->

<!--
Approving this PR does not by itself authorize publishing, tagging, a visibility
change, lifecycle promotion, or any other irreversible action. Those are
recorded separately; see RELEASING.md.
-->
