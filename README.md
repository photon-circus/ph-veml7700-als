# ph-veml7700-als

Async, `no_std`, allocation-free Rust driver for the Vishay VEML7700 ambient-
light sensor.

> [!WARNING]
> **Lifecycle:** Incubating.
> **Distribution:** Unpublished; the candidate version is
> `0.1.0-incubating.1` and the manifest retains `publish = false`.
> **Model conformance:** An independent I²C-level model covers `probe` and one
> successful `measure_once` path only. All other public operations are outside
> the current model claim.
> **Physical evidence:** None. No reviewed physical or calibrated-optical
> evidence exists. Evidence applies only to the named operations, and eventual
> publication would not imply hardware qualification.

## Responsibility

The crate owns complete single-device operations over caller-provided async I²C:

- explicit snapshot versus fresh-measurement semantics;
- fixed-address identity checks and little-endian register framing;
- conservative wake/integration timing with provenance;
- restoration-aware fresh capture;
- typed power-saving and threshold-monitor domains;
- raw ALS and white counts; and
- integer, nominal ALS count-to-lux scaling.

## Scope boundaries

The crate does not own an MCU, HAL, executor, bus recovery, optical fixture,
window/diffuser compensation, source-spectrum correction, empirical high-lux
correction, calibrated metrology, automatic ranging, or a fictitious interrupt
pin. The VEML7700 threshold output is polled through its status register.

Nominal lux is a datasheet-table conversion, not a claim about illuminance at a
finished product's aperture.

## Repository layout

```text
crates/veml7700/         driver and host-side tests
crates/veml7700-model/   independent device behavioral model (bounded slice)
docs/                    device, API, architecture, invariant, and test contracts
scripts/ci.sh            canonical bounded local verification
tools/check.ps1          PowerShell launcher for the same gate under Git Bash
```

Start with [the documentation index](docs/README.md).

## Local verification

Run `./scripts/ci.sh` under Git Bash or another POSIX-compatible shell, or
`./tools/check.ps1` from PowerShell, which locates Git Bash and runs the same
script. A green gate establishes agreement with the implemented host contracts
only; it does not establish physical-device or calibrated-optical behavior.

A GitHub Actions workflow runs the same script with `CI_PROFILE=bounded`, which
skips the dependency policy, four of the five bare-metal targets, and packaging,
and prints each skip. It is contributor feedback, not the release gate. It is
currently dispatch-only; see the deviation below.

## Standards deviations

Recorded under Photon Circus Repository Standards §19.

### Hosted CI does not run automatically

- **What differs:** organization standards §14.3 expects released public
  software to provide bounded GitHub Actions CI. The workflow exists and is
  reviewed, but `.github/workflows/ci.yml` is `workflow_dispatch` only; its
  `pull_request` and `push` triggers are commented out.
- **Why:** runs in a private repository bill against the organization's Actions
  allowance, which is exhausted. An automatic trigger produces a run that fails
  before it starts, marking every pull request red without saying anything about
  the code. Standards §14.2 names this case directly, allowing a private
  repository's hosted workflow to be "minimal, manually dispatched, or disabled"
  when it would consume shared capacity without proportional value.
- **Risk:** contributors get no automated feedback, and the workflow itself is
  unverified — GitHub has parsed it and built its job graph, but no job has ever
  executed. Expect the first real run to need a fixup. Reviewers must rely on a
  full local gate result pasted into the pull request.
- **Temporary or intrinsic:** temporary. Actions on standard runners is free for
  public repositories, so the constraint disappears at the visibility change.
  Restoring the two triggers is part of that change.
- **Approved by:** the repository owner, as a deliberate cost decision.

### Default-branch protection is not enabled

- **What differs:** §15.2 expects protection for Incubating repositories once
  reliable CI exists.
- **Why:** GitHub rejects branch protection for private repositories on the
  current plan.
- **Risk:** `main` accepts force pushes and deletion until the visibility change.
- **Temporary or intrinsic:** temporary; applied at the visibility change.
- **Note:** §15.2 requires an aggregate hosted check only "when an affordable
  hosted workflow exists", so the protection profile applies without it in the
  meantime.

## Publication status

The package retains `publish = false`. Release preparation, repository
visibility, and crates.io publication are separate maintainer-controlled steps
described in [RELEASING.md](RELEASING.md). Model completeness, physical
evidence, hardware qualification, and `ph-hil` adoption are not publication
prerequisites; they limit only the claims they support.

## License

Licensed under the [MIT License](LICENSE).
