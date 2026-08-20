# Contributing

This page contains contributor policy. Automated agents use `AGENTS.md` for a
short operational subset.

## Scope before artifacts

For a new driver/model pair, first read the pinned sources to identify the
peripheral's externally observable capabilities. This is a short, disposable
survey for selecting product scope. Do not infer a driver API, driver policy,
model state machine, oracle result, conformance claim, or CI rule during that
pass.

After the maintainer selects a product slice, add stable evidence propositions
only when a concrete driver, model, conformance trace, approved hardware
investigation, or reported bug needs them. An implementation roadblock is a
valid demand signal. The maintained consumer precedes the maintained evidence
artifact.

The capability survey is not a maintained contract, completeness checklist, or
backlog. Do not preload the registry with every source fact or absence and then
invent consumers, tests, hardware work, issues, decisions, or CI to justify it.

This repository owns an async, allocation-free `no_std` VEML7700 driver and
independent test model. MCU examples, board support, fixtures, and speculative
hardware infrastructure are out of scope.

## Setup and verification

| Tool | Version source |
| --- | --- |
| Rust 1.92.0, clippy, rustfmt, five targets | `rust-toolchain.toml` |
| `cargo-deny` 0.20.2 | asserted by the gate |

```sh
git clone https://github.com/photon-circus/ph-veml7700-als
cd ph-veml7700-als
cargo fetch --locked
cargo xtask ci
```

`full` is the default and authoritative pre-PR gate. `bounded` is hosted
feedback and names every skipped step; a skip is not a pass. `release` adds
clean-tree and artifact identity checks but performs no publishing action.

[`docs/VERIFICATION.md`](docs/VERIFICATION.md) owns the test-layer boundaries,
exact driver-versus-model inventory, and profile details. A
maintainer-authorized check belongs in `cargo xtask ci`, not a second workflow
implementation. Do not widen CI with prose classifiers or evidence-sufficiency
heuristics.

## Updating shared evidence

[`docs/HARDWARE_CONTRACT.md`](docs/HARDWARE_CONTRACT.md) is the only prose
authority for device and documentary propositions. Every `S-nn` permanently
names one atomic, scoped proposition. Evidence may be appended as supporting,
refuting, conflicting, or not resolving it; the proposition itself is never
rewritten. A changed or split proposition receives a new ID and leaves the old
one as a resolvable tombstone.

The registry grows on demand. A retained legacy row marked **not currently
relevant** has no current driver, model, conformance, approved-hardware, or
reported-bug consumer. It is not a backlog or coverage gap. Never add a new row
already known to lack such a consumer.

One evidence correction is one bounded transaction, even when several layers
must change:

1. update the proposition's evidence and state once;
2. enumerate every citation to its ID;
3. record the driver outcome: change or no change;
4. independently record the model outcome: change, no change, or unsupported;
5. record any approved hardware outcome; and
6. update conformance only where supported driver and model surfaces overlap.

The separate outcomes are reviews within the transaction, not automatic
follow-up issues. Create an issue, decision entry, audit, hardware task, or CI
rule only when the maintainer explicitly requests that artifact. Unrelated
observations stay in the handoff and are not blockers by default.

A suspected device-behavior bug starts by identifying the exact existing
`S-nn`. If none exists, add the smallest truth-apt proposition rather than
stretching an identifier or pasting a surrogate. Then assess driver, model, and
hardware consequences independently. A pure software or API defect may have no
device proposition.

## Evidence language and citations

Evidence polarity and knowledge state are separate:

- **Positive evidence** supports the proposition.
- **Negative evidence** refutes a device proposition or supports a located
  documentary omission. Source silence says nothing by itself about silicon.
- **Undefined** means accumulated evidence does not determine a device
  proposition. It is not evidence and does not imply the opposite behavior.

Physical observations record units or lots, revision, reset history, voltage,
temperature, procedure and tool commit, plus a durable raw-artifact citation or
digest. Conflicting observations remain visible with their scopes. Model,
scripted-transport, and code-reading results are labelled as such and never
promoted to physical evidence.

A source statement may be a characterized limit or vendor guidance; a citation
does not upgrade one into the other. A silence claim requires a located
negative: pinned document revisions and the exact sections searched. See
[`D-032`](docs/DECISIONS.md#d-032-a-silence-claim-needs-a-located-negative).

Outside the registry, do not quote or paraphrase a proposition, vendor
coordinate, or hardware observation. State only the local component consequence
and cite every applicable `S-nn` in that paragraph, table row, or code comment.
The stable ID is the shared meaning; each registry row is directly linkable as
`#s-nn`. `cargo xtask ci` checks only that IDs are unique and references resolve;
review owns semantic correctness.

## Document authority

| Document | Authority |
| --- | --- |
| `docs/HARDWARE_CONTRACT.md` | descriptive shared propositions, evidence, and evidence state |
| `docs/vendor/README.md` | vendor source identity, retrieval facts, and digests |
| `docs/DRIVER_CONTRACT.md` | normative cross-cutting driver ownership and guarantees |
| generated Rustdoc | exact public API behavior, errors, timing, and cancellation |
| `crates/veml7700-model/README.md` | normative model scope and behavior |
| `docs/VERIFICATION.md` | contributor procedure and exact conformance inventory |
| `docs/DECISIONS.md` | non-normative durable rationale only |
| `RELEASING.md` | release procedure and reserved decisions |
| `CHANGELOG.md` | released versions and the current unreleased user-visible surface |

Other documents give only the minimum audience-specific summary and link to the
authority. A behavior change updates its protecting tests and authority in the
same pull request. An evidence correction does not require new durable rationale.

## Pull request workflow

1. Keep one bounded concern per pull request. One evidence correction remains
   one concern across every directly affected layer.
2. Use a governing issue or decision record only when the maintainer requested
   it.
3. Name affected `S-nn` IDs without reproducing their propositions, and record
   driver, model, and approved-hardware outcomes separately.
4. Put tests in the layer that can establish the claim; follow
   `docs/VERIFICATION.md` and keep the model independent of driver internals.
5. Update `CHANGELOG.md` only for user-visible behavior.
6. Run the full gate and paste its final line.

Do not publish packages, add credentials, or create tags or releases. The crate
being on crates.io does not make the next version routine; each release remains
a separate maintainer decision in `RELEASING.md`.

Do not commit vendor PDFs. Redistribution permission has not been established;
`docs/vendor/README.md` records source identity and digests.

## Licensing

The project is MIT licensed. By contributing, you agree that your contribution
is licensed under the [MIT License](LICENSE). Contribute only material you have
the right to license; do not paste vendor source text or incompatible code.
