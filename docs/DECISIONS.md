# Decision log

> **Authority: non-normative rationale.** Why things are the way they are,
> including decisions later superseded. Superseded entries are marked, not
> deleted: the reasoning that was rejected is part of the record. Nothing here
> is an acceptance contract.

## D-001 — Direct fixed-address I²C facade

`Veml7700<I2C>` owns the bus directly. The supported device has one fixed
address and no transport variation requiring an internal abstraction.

## D-002 — Preserve concrete bus errors

Errors retain the HAL error plus semantic operation/register/stage context.
Only address NACK is classified as absence during probe.

`probe` reports through `ProbeError` instead, because address NACK means absence
only there. It therefore carries no `Operation`, so `Operation` has no `Probe`
variant: a variant no driver path can construct is surface a caller could match
on and never reach.

## D-003 — Strict low-byte-first codec

All 16-bit register transfers use little-endian wire order, protected by exact
transaction tests.

## D-004 — The starting configuration is explicit policy

`MeasurementConfig::maximum_range_start()` is a value constructor, not hidden
driver state or automatic ranging.

**Superseded name.** It was `safe_bright_start()` at gain ×1/8 and 100 ms, which
saturates at ~35 232 lx — under direct sunlight, while being named for safety in
bright light. The name promised range the configuration did not have.

The replacement is source-backed rather than merely renamed: the vendor says to
start at the lowest gain for unknown brightness, and that an integration time
below 100 ms may be needed to show such a value. ×1/8 with 25 ms reaches
~140 926 lx, the widest the part offers. The gain was always right; the
integration time and the name were not.

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
committing vendor PDFs. The untracked claim is enforced by the canonical gate
rather than by `.gitignore` alone.

## D-017 — Local bounded validation

There is one canonical gate, `scripts/ci.sh`, and it has three profiles. `full`
is authoritative for ordinary work and is what a maintainer runs. `bounded` is
the subset hosted CI runs; it drops the checks needing an extra binary or
substantial runner time and reports each as an explicit skip, so a green hosted
run can never be mistaken for a green release gate. `release` is `full` plus
artifact identity: it refuses a dirty worktree, packages without
`--allow-dirty`, and records the source-to-archive relationship as evidence.

`release` is a superset rather than a replacement. `full` deliberately keeps
packaging permissive so a work-in-progress tree stays checkable, which is the
common case; demanding a clean worktree for every routine run would make the
gate hostile to ordinary development. The two therefore differ only in what they
demand of the tree, not in what they verify about the code.

Only the PowerShell launcher is retained beside the script, because locating Git
Bash on Windows is real work and a POSIX passthrough wrapper is not.

The hosted workflow is prepared before the repository is public, but is
dispatch-only until then; its automatic triggers belong to the visibility
change. A dispatched failure is noted and investigated, but the full local run
is the only authority. Generated pack inventories remain rejected.

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

## D-021 — No documentation index

`docs/README.md` was deleted because it restated the AGENTS.md document table
and went stale; the root README links the contracts directly.

## D-022 — One version across the product crates

Both **product** crates — the driver and the model — carry the same
lifecycle-matching prerelease, currently `0.1.0-incubating.1`, and the canonical
gate fails if they diverge.

`ph-veml7700-als-conformance` is deliberately outside this. It pins the `0.0.0`
sentinel and does not inherit the workspace version, because it has no release
boundary: no consumer can observe it, and a release bump should not touch it.
Letting it inherit would be the natural-looking edit and would quietly enrol a
test harness in the product lifecycle, so the gate asserts both the sentinel and
`publish = false` rather than trusting the manifest to stay that way.

The Peripheral Driver Profile attaches its prerelease rule to the driver
package, so `ph-veml7700-als-model` could have kept an ordinary `0.1.0`. Aligning
was chosen instead because the two crates share one repository, one release
boundary, and one review cycle: a reader comparing them should not have to work
out which convention applies to which, and a second convention is a place for
drift to hide.

The gate no longer repeats the exact version. It reads the driver manifest,
requires the model manifest to match, and asserts only that the version carries
an `-incubating.N` prerelease identifier. A version bump therefore edits the
manifests, and the gate keeps verifying the declared distribution state without
holding another copy of the literal.

## D-023 — Undeclared reset values stay unavailable

The hardware-contract register map records which words have a source-declared
reset and which do not. Configuration (`0x00`) resets to `0x0001` and
power-saving (`0x03`) to `0x0000`. Thresholds, ALS, white, and threshold status
are **not declared by sources**. The ID word is a source-declared identity,
not a POR field.

Determinism does not authorize a convenient initial value for that undeclared
observable state. The independent model leaves those registers unavailable
until an explicit input establishes them: a threshold write, a completed
conversion, or a completed ALS refresh while the monitor is enabled. Inventing
`0x0000` for threshold status would make a fresh `read_threshold_status()`
decode to both flags clear with no source backing.

One residue is unavoidable and is therefore declared rather than hidden. Because
this model asserts threshold flags and never clears them, a refresh that
qualifies nothing cannot prove a flag is clear, so the establishing refresh
cannot derive the whole `0x06` word from transitions alone. The model declares
that construction represents a device with no prior qualification; the
observable consequence is that an unqualified flag reads clear after the first
monitored refresh. The alternative — tracking each flag's knowledge separately —
would leave `0x06` unavailable whenever a flag never qualifies, which is the
ordinary case, and would make the driver's polled-status path untestable for no
gain in evidence.

## D-024 — Error taxonomies are open, device domains are closed

Every public error type in both crates is `#[non_exhaustive]`. Every public
type that enumerates a device domain is deliberately not.

Error taxonomies grow as the driver and model grow. `Unsupported` alone gained
nine variants across five commits before the first release, and `Operation`,
`BusContext`, `MeasureStage` and `ThresholdMonitorStage` extend with each new
operation, register and stage. Without `#[non_exhaustive]` each of those
additions breaks a downstream exhaustive `match`, so ordinary additive work
would force a breaking release. The attribute is free before publication and
cannot be added afterwards without causing the break it prevents.

The device value types — `Gain`, `IntegrationTime`, `Persistence`,
`PowerState`, `ThresholdMonitorState`, `PowerSavingMode`, and
`MeasurementPairCoherence` — stay exhaustive. They enumerate a fixed domain
taken from the datasheet, not an open taxonomy. A caller that handles every
gain should keep getting a compile error when it misses one, and the device will
not grow a fifth gain. If a future part does, that is a new contract and a
deliberate breaking change rather than routine growth.

The three decode errors are open with the errors, not closed with the domains
they describe. `ConfigDecodeError`, `PowerSavingDecodeError` and
`ThresholdStatusDecodeError` currently track reserved-bit rules that are as
fixed as the register map, but they are error types: a caller matches them to
report, not to make a decision per variant, so exhaustiveness buys the caller
little and costs a break if the contract ever distinguishes a new decode
failure.

`ThresholdMonitorError` is a struct rather than an enum and carries the
attribute for the same reason: it is produced by the driver and read by the
caller, so preventing downstream literal construction costs nothing and lets it
gain a field later. The obligation it places on a caller is different from the
enums, though, and the API contract says so: a struct pattern must contain `..`,
and a wildcard match arm does not satisfy that. Ordinary field access is
unaffected, which is how callers actually read this type.

Result and snapshot types with public fields, such as `FreshMeasurement` and
`ConfigurationSnapshot`, are deliberately left alone here. Whether a caller
should be able to build one is a separate question from error growth, and
answering it under this decision would be scope creep.

## D-025 — No CODEOWNERS while there is one maintainer

**Date:** 2026-08-14 **Status:** Current

The organization standard requires `CODEOWNERS` only where there are multiple
maintainers or separated sensitive ownership, and asks a solo-maintained
repository to record whether it adds value. It does not, yet.

`CODEOWNERS` expresses that a particular person must review a particular path.
With one maintainer every path resolves to the same person, who is also the
author of nearly every change. The file would not add a second reviewer; it
would auto-request review from the author, and advertise a separation of
ownership that does not exist. Both are worse than saying plainly that this is a
single-maintainer project.

The boundaries a `CODEOWNERS` file would have protected — the source contracts,
the release manifests and workflow, and the security policy — are protected
instead by mechanisms that work with one maintainer: the canonical gate, the
required aggregate `ci` check once `main` is protected, and the rule that a
contract change is reviewed as a behavior change.

Add `CODEOWNERS` when a second maintainer joins, covering at minimum
`docs/HARDWARE_CONTRACT.md`, `docs/vendor/`, `scripts/ci.sh`,
`.github/workflows/`, the crate manifests, and `SECURITY.md`. That is the point
at which it starts routing review rather than describing a single person.

## D-026 — Reconfiguration is shutdown-first

**Date:** 2026-08-14 **Status:** Current

Every mutating operation writes the shutdown bit before changing any other
field, and returns the device to active last. `set_power_state` is not a
reconfiguration and is unaffected.

This is not a defensive choice. The vendor's own software flow sets `ALS_SD = 1`
before any reconfiguration, changes gain or integration time while shut down,
and clears `ALS_SD` afterwards. `docs/HARDWARE_CONTRACT.md` `S-19` records it as an
owner-verified row. It is a positive requirement, not an absence of permission
to write while active, and that distinction is what settled the question.

The driver and the independent model disagreed here: the driver accepted
monitor-disabled active starting states, and the model rejected changed or
repeated active configuration. **The disagreement was resolved against the
driver.** The model was correct and was not relaxed to admit the sequence it had
been rejecting — which is the outcome an independent oracle exists to produce.
Relaxing it would have destroyed the only evidence that the two derivations
disagree at all.

Three costs are accepted deliberately:

1. **More transactions.** `set_measurement_config` and `set_power_saving` take
   three writes instead of one when the device is active. A single write that
   moved both the shutdown bit and another field would be cheaper and is exactly
   what the sources forbid.
2. **A different failure state.** Because shutdown comes first, a failure part
   way through can leave an originally active device shut down. The contract
   states the read-back obligation rather than pretending the operation is
   atomic; the transport cannot make it so.
3. **A new public stage.** `MeasureStage::EnterShutdown` and
   `ThresholdMonitorStage::EnterShutdown` name the pre-reconfiguration write, so
   a failure there is attributable instead of folded into a later stage. Both
   enums are `#[non_exhaustive]`, so this is additive. See D-024.

Entering shutdown first also fixed recovery, which was not the goal but is the
stronger argument. Previously an active start failed *and* its restoration
failed, because restoration wrote the power-saving register to a still-active
device. Every stage after the first now runs on a shut-down device. The one
remaining case — a failure of the shutdown write itself — reports without
attempting restoration, because nothing has been mutated and the generic
sequence would commit the very active write this decision removes.

Two writes remain legal while active because both are transitions rather than
reconfigurations: setting the shutdown bit alone, and disabling an enabled
monitor alone. They cannot be combined. Re-arming an enabled monitor on an
active device therefore shuts down first with the monitored domain intact, then
disables the monitor while shut down.

## D-027 — Cancellation is documented, not defended

**Date:** 2026-08-14 **Status:** Current

No async operation is cancellation-safe. Dropping a future does not undo what it
has already done, and the public documentation says so at every await boundary
rather than implying otherwise.

Making them safe was considered and rejected. A driver cannot run cleanup during
a drop: `Drop` is synchronous, the bus is `async`, and there is nowhere to await
the restoring writes. The options that do exist are worse than the limitation.
A cleanup guard would need a blocking bus it does not have. A background task
would need an executor this crate refuses to depend on. Both would trade an
honest limitation for a hidden one.

So the driver states the boundary behavior exactly and gives a recovery
procedure using public operations only: read the registers back, shut down to
stop an abandoned conversion, then reinstate the domain. Read-back is the
instruction because inference is unsound — the state after a drop mid-write is
genuinely ambiguous.

The measurement delay is called out separately in the documentation. It is by
far the longest suspension, so a timeout or `select!` lands there most often,
and it leaves the sensor awake and converting in a domain the caller did not ask
to persist. The guidance is to bound the operation with a shorter integration
time rather than by racing the future.

**A failed write is not a rejected write.** An I²C error can mean the byte never
arrived, or that it arrived, took effect, and the acknowledgement was lost. The
transport cannot distinguish them, so no error type in this crate reports a write
as rolled back or not applied. `ThresholdMonitorError` carries `confirmed` — the
last stage that definitely landed — separately from `stage`, whose commit status
is unknown. A caller can therefore act on what is certain without being told
something the bus never established.

Cancellation is tested by sequencing, not by state. A pending-capable transport
parks on a chosen operation; the test polls once, drops, and asserts exactly
which transactions were issued. Whether the device physically committed the
transaction in flight is unknowable in a scripted harness and is asserted
nowhere. That boundary is the same one the model respects, and it is why these
tests live beside the scripted transport rather than in the independent model.

## D-028 — One authority per subject

**Date:** 2026-08-14 **Status:** Current

Every maintained document states its authority in a header: normative contract,
evidence record, contributor procedure, or non-normative rationale. Before this,
`CONTRIBUTING.md` called everything under `docs/` normative, which gave
architecture prose and historical decisions the force of an acceptance contract
they were never written to carry.

Three consolidations follow from applying that consistently.

`ARCHITECTURE.md` and `API_CONTRACT.md` became `DRIVER_CONTRACT.md`. They
described the same thing from two angles — ownership and dependency direction in
one, semantic promises in the other — and a reader had no way to know which
governed when they disagreed.

The handwritten public-signature inventory is deleted rather than moved. The
compiler and rustdoc own signatures; a prose copy can only drift, and it had.
What survives is what neither can express: what the surface *means*.

`DOCUMENTATION_STANDARDS.md` was eleven lines, six of which `CONTRIBUTING.md`
already carried after #33. The seventh — that examples compile as doctests —
is review-blocking, so it became I-28. A document too small to justify its own
authority is a place for drift to hide.

`TEST_PLAN.md` became `VERIFICATION.md` and defers to the packaged coverage
matrix rather than restating it. A plan describes intent; this describes what
each layer establishes, which is the useful thing.

**Vendor provenance moved to `docs/vendor/README.md`.** `AGENTS.md` previously
named the model README canonical for digests, which put source identity under a
document whose subject is the model. A digest is a fact about a retrieved file
and belongs with the retrieval record.

`docs/README.md` remains absent, per D-021. An index of seven documents is a
seventh thing to keep true.

## D-029 — Assumptions about silicon are declared, not left open

**Date:** 2026-08-14 **Status:** Current

A contract row that the sources do not settle is in one of two situations, and
conflating them wastes effort and hides risk.

Some facts are simply **not on the page consulted**. Another passage may state
them, so the row stays provisional and the work is more reading.

Others are **not knowable from the documents at all**. The driver still has to
behave one way or the other, so it rests on an assumption — and no amount of
reading resolves it. Leaving such a row looking like an unread one implies a
search that will eventually succeed. It will not.

These rows are therefore marked **Assumption**, and each states three things:
what the driver assumes, where that assumption is expressed in code, and what
observation would settle it. That last part matters most: an assumption without
a stated test is indistinguishable from a guess, and it gives a future
hardware-in-the-loop effort nothing to start from.

The first is refresh-time independence from ALS gain. The sources publish the
relation at gain ×2 only. `nominal_refresh_time_ms` takes no gain argument and
the model's `refresh_interval_ns` inherits the same shape, so both behave as
though independence holds. Measuring the refresh interval across all four gains
at one integration time and power-saving mode would settle it in an afternoon
with hardware, and cannot be settled without.

This does not change the repository's evidence posture. Physical evidence
remains none, and an assumption declared honestly is not evidence — it is a
named place where evidence is missing, which is strictly better than the same
gap left implicit in a function signature.

## D-030 — Undefined device behavior is allocated, not split evenly

**Date:** 2026-08-14 **Status:** Current

D-029 says an unresolvable row is declared rather than left open. It does not
say *who* declares it. The driver and the model face the same silence for
different reasons, and giving them the same answer is what produces a false
oracle.

The allocation turns on one question, asked separately for each component:
**does it need the fact in order to function at all?**

The **driver** acts defensively wherever the answer is no. A promise it cannot
back is worse than no promise, because a caller cannot tell the difference until
it is wrong in the field. Where the answer is yes — a wait must be *some*
duration, a register must be decoded *some* way — it assumes, and says so at the
constant, in the contract row, and in the public documentation of anything whose
behavior depends on it.

The **model** declares undefined by default. Its whole value is being an oracle
derived independently of the driver; a model that invents plausible behavior
still produces agreement, and that agreement means nothing while looking exactly
like evidence. `TransportError::Unsupported` is the correct answer to a question
the sources never settled. Where the model genuinely cannot run without the fact
— construction needs some register value to represent a power-on device — it
assumes, and the assumption is tabulated in the model README with its code site.

The open rows resolve differently under this rule, which is the point:

| Row | Driver | Model |
| --- | --- | --- |
| Register `0x03` reset value | Defensive — reads before acting, so it needs nothing | Assumes, to construct a power-on device |
| Refresh time ⊥ ALS gain | Assumes — a cadence must be computed | Assumes — inherited, same silent source |
| Persistence qualification rule | Defensive — no driver logic depends on it | **Declares undefined** — nothing forces it to guess |

The ±30 % integration tolerance was a fourth row here, allocated as *both
assume*. It left the table under D-032: the sources do state the figure, so
there was no silence to allocate. Its removal changes no behavior — the driver
still waits 130 % and the model still completes at 130 % — which is what makes
it a clean illustration of the boundary. This rule governs what to do about
silence; it has nothing to say once the silence turns out not to be there.

The last row is the one that shows the rule working. The model had implemented
consecutive counting with reset on any non-qualifying refresh, and driver-model
conformance then *appeared* to establish persistence semantics. It did not: the
driver only programs the field, so the trace confirmed a register write while
reading like a behavioral result. Nothing about the model requires a
qualification rule, so it declares undefined instead.

**That row's premise was later corrected, and the outcome held** (#73). This
decision originally rested on the sources stating no qualification rule at all.
They state part of one: application note 84323 gives the counting condition in
necessary form — a flag is set *only when* `ALS_PERS` measurements stay above or
below the threshold. What remains unstated is whether meeting that condition is
sufficient, and what a non-qualifying measurement does to a partial run.

The allocation is unchanged, because predicting an assertion needs both of those
and the model still has neither. It is a better example for having survived the
correction: the rule allocates whatever silence remains, and it does not depend
on the silence being total. A decision whose worked example turned out to be
half wrong, and whose outcome did not move, is worth more than one that was
never tested.

A consequence worth stating plainly: this makes the model's covered surface
smaller. That is the correct direction. A narrower oracle that is sound beats a
broader one that manufactures agreement.

## D-031 — The model's time domain is bounded, and rejects rather than batches

**Date:** 2026-08-14 **Status:** Current

`Veml7700Model::advance` accepts one hour of virtual time per step and panics
above it. `MAX_ADVANCE` is a **model-domain constraint, not a performance
guard**, and the distinction decides whether raising it is safe.

### Why a bound exists

`advance` loops once per refresh event. The shortest recurrence the model can
select is 130 % of 25 ms — about 32.5 ms — so a `u64::MAX` nanosecond input
implies roughly 568 billion iterations. That is not an error a suite reports; it
is a hang, which is the worst available outcome. A test that hangs gives a
maintainer no argument, no line number, and no failing assertion.

### Why reject rather than batch

Batching event-free recurrence was the alternative, and it is rejected as a
correctness risk disguised as an optimization.

Collapsing N refreshes into arithmetic requires proving that N refreshes are
observationally equivalent to some closed form. Today they nearly are — the
held pair is constant across them and threshold status is monotonic — but "nearly"
is doing real work in that sentence, and it would stop being true the moment the
model gains a per-refresh behavior that is not idempotent. The equivalence proof
would then be wrong silently, in the direction of the model agreeing with the
driver about a timeline neither had actually walked. That is the failure mode
this repository keeps finding, and it is not worth reintroducing to make a test
that should not exist run faster.

The literal loop is slow only for inputs that are already defects.

### Why one hour

It is generous against every cadence the model can select. The longest is 800 ms
integration plus a 4 s Mode 4 refresh, so an hour is roughly 750 refreshes; at
the shortest cadence it is about 110,000 iterations, which is microseconds of
work. No legitimate scenario in this slice needs a single step that long.

The number is not derived from the device and claims nothing about it. It is the
point past which a single step is more likely a units mistake — nanoseconds
supplied where microseconds were meant — than a scenario.

### What raising it would mean

Raising the bound is safe for iteration cost up to roughly a day. Beyond that,
reconsider the loop rather than the constant. What raising it does **not** do is
change any device claim: the bound is a statement about what this harness will
accept, not about how long a VEML7700 runs.

The same constant bounds the injected white-channel phase offset, which is what
makes the white wake edge provably non-overflowing rather than saturating. Any
change to `MAX_ADVANCE` has to keep that argument intact — the wake edge is
`conversion_bound + offset`, and the checked add there exists to stay loud if
this reasoning is ever loosened without being rechecked.

### Rejection happens before mutation

A caller that catches the panic observes an unchanged model. Without that, a
rejected advance would be indistinguishable from one that ran partway and
stopped, and the model's own tests could not tell the difference either.

## D-032 — A silence claim needs a located negative, not an argument

**Date:** 2026-08-14 **Status:** Current

The repository stated that no vendor document gives an integration-time
tolerance, called the ±30 % figure third-party in origin, and explained at
length why no such passage could exist. The passage exists. Application note
84323, Revision 06-Mar-2025, page 4, section *Command Code ALS_IT*, `Remark`:
"For the integration time a tolerance of ± 30 % can be assumed. This tolerance
should also be considered during the read out of the measurement results." It is
in a pinned source, under the digest the contract already anchors to.

### How a careful process produced a false negative

Not by skipping the reading. By reasoning about the **kind** of fact instead of
about the document. Integration intervals are counted off an internal
oscillator; an oscillator tolerance is a process characteristic; vendors do not
publish process characteristics for parts like this. Each step is sound, and the
conclusion — that further reading was pointless — is what closed the search.
Once a row says *further reading cannot close this*, nobody reads further. The
argument became load-bearing precisely where it was least examined.

The failure mode is specific to a D-029 Assumption. It is the strongest claim
this contract can make about a source: not "we did not find it" but "it is not
there and cannot be". That claim also suppresses the work that would refute it,
which makes it the one row type that must not rest on inference.

### The rule

An Assumption declaring that reading cannot close a row must record a **located
negative**: which document, which revision, which sections were read and found
silent. A reader must be able to check the claim by opening the same pages. An
argument from the nature of the quantity may accompany that record; it may not
substitute for it.

The rule applies to the rows that already exist, not only to future ones — a
rule unmet on the day it is written is advice. Both surviving Assumptions carry
located negatives as of this decision: refresh-gain independence in §6 and the
`0x03` power-on word in §4.

Applying it immediately earned its keep. The `0x03` row had said *no passage
declares it*; the application note in fact states that bits 2:1 come up as mode
1, which covers the `PSM` field and leaves only `PSM_EN` undeclared. That is the
same shape of error as the one this decision was written for, found the first
time the rule was exercised rather than several revisions later. The row's
supporting sentence is corrected here; narrowing its subject is a contract-state
change, so it is #71.

### The rule is about absence claims, not about Assumption rows

A third instance widened it. §2 recorded, inside a **verified** row, that "1.7 V
appears nowhere in the source". It appears in three places, including a column
of the datasheet's page-one Product Summary named `I²C BUS VOLTAGE RANGE` (#67).
No Assumption was involved, so the rule as first written would not have reached
it.

It is therefore stated generally: **any claim that a source does not say
something needs a located negative**, wherever it appears and whatever the row's
checkbox state. A checked row is not a safer place to assert absence; it is a
more dangerous one, because the checkmark reads as confirmation of everything in
the row rather than of the fact it records.

That instance also shows how the error propagates. #67's predecessor #54 had
enumerated "1.7 V is from a different parameter" as one of three possible
outcomes — the correct one. It was passed over because the search stopped at the
passage the dispute quoted, and the resulting absence claim then read as settled.
A dispute about one passage is not a review of the document.

### What the audit found

Three instances in quick succession made the rate the point, so every absence
claim in `HARDWARE_CONTRACT.md` was audited against both pinned documents rather
than waiting for a fourth to surface on its own.

One more was false: §9 said no reviewed passage states the persistence
qualification rule, and application note 84323 printed page 16,
`INTERRUPT HANDLING`, states the counting condition. That is #73, and it matters
beyond the row — this is D-030's worked example, so a decision illustrated by
"the sources are silent here" rests on a premise that was not checked.

The rest held: the clock-mode split, the undeclared reset values, Table 1's
silence on the power-on word, the missing 25/50 ms refresh rows, the absent sign
bit, and the absent flag-clearing rule are all confirmed. They now carry located
negatives naming the sections read.

Three limits of that exercise are worth recording, because a future reader will
otherwise over-trust it. Presence is provable and absence is not: every positive
finding here quotes text, while every negative is a search over sections that
were read, which is why the rule asks for sections rather than for a global
negative — and why each backfilled row says that absence outside those sections
is not claimed. The searches ran over machine-extracted PDF text, which can drop
glyphs set in figures.

The third limit was found the hard way, and it applies to searching *this
repository* rather than the vendor documents. The first sweep for a false claim
missed the packaged crate README, because the phrase wrapped across a line and
the search was line-oriented. Every tracked document here is hard-wrapped, so a
grep for any phrase longer than a few words is unreliable by construction —
**normalize whitespace before matching.** The missed copy was crate
documentation, so it was the one site that reached consumers.

The physics argument here was, in fact, still correct about the thing it
described — Vishay publishes no oscillator accuracy, and the datasheet shows the
oscillator only in the block diagram. What it was wrong about is that this
implies vendor silence on integration-time tolerance. An application note gives
design guidance, which is a different act from specifying silicon, and this
repository had no category for it.

### Vendor-stated is not characterized

That category now exists, and the correction stops there. The vendor writes
"can be assumed", which is a design allowance, not a min/max across process,
voltage, and temperature. So `INTEGRATION_TOLERANCE_PERCENT` is now sourced but
is still not a guarantee, and a spread wider than ±30 % still breaks the
freshness guarantee silently. Recording a citation raises the provenance of a
claim, never its strength — see the evidence-language rules in
`CONTRIBUTING.md`.

### What did not change

The driver waits 130 % of the selected integration time. The model completes at
130 %. No test moved. The repository had the right number and the wrong account
of where it came from, and under this repository's posture the second is the
part that matters: a correct value carried by a false provenance claim is
exactly what the contract exists to prevent, and it survived several revisions
of deliberate review.

Consequently #58's integration-time observation is no longer the means of
discovering an unpublished figure. It is optional characterization — evidence
about parts on a bench, against a tolerance the vendor already states.

## D-033 — Source claims are cited, not restated

**Date:** 2026-08-15 **Status:** Current

D-032 made absence claims checkable. It did nothing about how many places carry
one, and that turned out to be the expensive part.

Four corrections in a row measured it: the ±30 % tolerance lived in six files
(#65), the `0x03` power-on word in four (#71), the persistence rule in **nine**
(#73). Two of the persistence sites were `pub` rustdoc, so a disproven claim was
being published rather than merely recorded. Correcting one claim meant finding
every copy by hand, and nothing detected a copy left behind.

They rotted in two ways, both observed. The claim itself went stale — seven
restatements of the persistence rule were wrong at once. And the *pointer* went
stale: two citations said §8 for a row that had moved to §9, and had read as
correct for revisions.

### Why a shared, stable evidence base is required at all

Conformance has two failure modes, and they destroy it from opposite directions.

**Share the interpretation, and the derivations collapse into one.** If the
driver and the model are told how to read a fact — or share the constant, codec,
or state machine that expresses it — their agreement is a tautology. Two
expressions of one derivation agree because they *are* one derivation. This is
the failure D-015 and D-030 guard, and it has been caught here in the field
(#56).

**Fail to share the evidence, and the derivations have no common baseline.** Two
independent readings only mean something if both read the same thing. Without one
stable record, a disagreement is uninformative — it could mean the driver is
wrong, or the model is, or that the two were working from different documents,
different revisions, or two copies of a sentence that had drifted apart. The
signal conformance exists to produce is *attributable* disagreement, and
attribution needs a fixed baseline.

So the evidence must be shared and stable; the interpretation must not be shared
at all. A **prescriptive** record fails the first way, because prescribing how to
interpret is how the interpretation gets shared. A per-crate, duplicated, or
drifting record fails the second.

Stability is why this is not just "put it in one file". The record is anchored to
digest-pinned sources (§1), its identifiers are permanent, and its rows change by
governance rather than in-place edit. Each of those exists so that a conformance
result means the same thing next year as it did when it was written.

### The rule

`docs/HARDWARE_CONTRACT.md` is the registry. Every row carries a stable `S-nn`,
the third instance of a convention this repository already runs twice, for
`D-nn` and `I-nn`. Everywhere else **cites the identifier**; a section number is
a position, and positions move.

`scripts/ci.sh` enforces three things, and the third is the one that pays for
itself:

1. identifiers are unique, so one number names one row;
2. every cited identifier resolves, so a retired or mistyped citation fails
   loudly instead of quietly naming the wrong claim;
3. any statement about what the sources do not say, outside the registry, cites
   an `S-nn`.

The third converts "find every copy" from a grep someone has to get right into a
mechanical list. `DECISIONS.md` and `CHANGELOG.md` are exempt: they discuss
claims historically, including claims since corrected, and rewriting history to
satisfy a citation rule would defeat the point of keeping it.

**What the gate cannot check is restatement.** It can tell that a paragraph
asserting source silence cites an identifier; it cannot tell whether the
paragraph then goes on to repeat the rule's words instead of stating this
repository's consequence. That distinction is semantic and stays a review
obligation. Several surfaces still restate today; converting them is the
remaining work, and the check is what makes that work enumerable rather than a
search.

Direct source citation, by contrast, **is** mechanical — document numbers,
revisions, and table names are literals. Twelve files currently carry them
outside the record. A fourth check can enforce that, and should; it is the
remaining part of #75 rather than of this decision.

Note what the two checks cover between them, because the asymmetry is an
artifact of implementation and not of principle. The existing check catches
uncited claims of **negative** evidence, which announce themselves in phrasing.
The proposed one catches uncited claims of **positive** evidence, which announce
themselves in document coordinates. Neither form is privileged; they are simply
detectable by different means.

The check reads paragraphs with whitespace collapsed rather than lines. Every
tracked document here is hard-wrapped, so a line-oriented search cannot see a
phrase spanning a line break — which is exactly how the packaged README kept a
disproven claim through an audit that believed itself exhaustive.

### Scope: this decision is about the record, not about reacting to it

Three subjects are easy to run together, and this decision owns only the first.

1. **Where the evidence lives, and how it is named and cited.** This decision.
2. **How the driver and the model each decide what a rule means for them** —
   whether to act defensively, assume, or declare undefined. That is D-030, and
   nothing here changes it.
3. **Whether a rule is established at all**, and what a claim of silence
   requires. That is D-029 and D-032.

The only thing this decision borrows from the others is a constraint it must not
break: driver and model implementations stay separate. If they share a constant,
a codec, or a state machine for a device fact, they share its mistakes and
conformance collapses into a tautology — two expressions of one derivation
agreeing because they *are* one derivation. `CONTRIBUTING.md` states that rule
and D-030 works out its consequences; centralizing the *record* must not become
an excuse to centralize implementations.

**With that fixed, the evidence itself must not be duplicated.** What the
datasheet says has no oracle value in duplicate: two copies of a quotation cannot
catch each other being wrong, only diverge. That is observed rather than
theoretical — the persistence rule was restated nine times and all nine were
wrong at once, and the copy that reached consumers was the one a hand search
missed.

The asymmetry is worth stating plainly, because it is what makes the two rules
compatible rather than contradictory. Two independent *derivations* can disagree,
and their disagreement is information — that is the whole mechanism of
conformance. Two copies of a *quotation* cannot disagree usefully; if they
differ, one is simply wrong. Duplicate what can be checked against reality;
centralize what can only be checked against itself.

An earlier draft of this decision said prose fragments should be per crate. That
was wrong, and it would have institutionalized exactly the rot this decision
exists to remove. A shared *sentence about the datasheet* creates no shared
behavior — prose does not execute — while a shared *constant* does.

### What the record is: agreed global facts, with their evidence

The global artifact makes **no interpretation**. It records what has been
**agreed** about global facts derived from the sources, and the evidence each
fact rests on. A row is therefore two things and no more:

- the agreed fact; and
- its **evidence**, which takes one of two forms and is equally evidence in
  either:
  - **positive** — the source states it, quoted with document, revision, page,
    and section;
  - **negative** — a located negative naming which document, revision, and
    sections were read, and disclaiming absence outside them.

Throughout this repository, *evidence* means both. Nothing here treats a
positive finding as evidence and an absence as its lesser cousin: a located
negative is a finding, it is what four corrections turned on, and the row it
supports carries it the same way. Where a distinction is needed, say **positive
evidence** or **negative evidence** rather than narrowing the bare word.

*Agreed* is doing real work in that sentence. A row is not one reader's reading;
it is the settled position of this repository, which is why changing a row is a
governance act with its own issue rather than an in-place edit, and why a row
carries its verification state on its face.

**The record is descriptive, never prescriptive.** It binds as a record — this
is what we agree the sources establish — and prescribes nothing. That is a
reclassification: the document called itself *normative* and *interpreted device
behavior*, which is precisely the prescriptive reading this decision rejects.
Prescription lives where it can differ per component: `DRIVER_CONTRACT.md` for
the driver, the model's own claim for the model.

Nothing else belongs in a row. Not what the driver should therefore do, not
whether the model can model it, not whether a figure is good enough to rely on.
Those are **reactions**, they differ per component by design, and a reaction
recorded as a fact is how a repository convinces itself that a judgment is
evidence.

Keeping the record reaction-free is also what lets one row serve two components
that answer it differently. If the row said what to do, it would have to say it
twice, and the two would drift — the same failure as restating a rule, wearing
different clothes.

There is one datasheet. From it this repository consolidates that evidence, and
**the consolidation is a single global artifact** — `docs/HARDWARE_CONTRACT.md` —
not a per-crate asset and not something a crate owns. `S-nn` is a global name,
valid from anywhere: driver, model, conformance, prose, or a future sibling
crate. Nothing scopes an identifier to a crate, because nothing scopes the
datasheet to one.

**The record does not yet meet this.** Many rows carry reaction alongside
evidence — "this driver applies it", "the driver does not rely on this", whole
D-030 allocation paragraphs. Separating them is #80, not this decision, and the
identifiers are what make it possible: a row can shed its reaction to
`DRIVER_CONTRACT.md` or the model README without breaking a single citation.

That settles a question this decision first got wrong twice. The answer is not
per-crate fragments, and it is not copies compared by the gate either. **It is
that no surface restates a rule.** Each states its own *consequence* and cites
the rule.

The driver and the model reach different consequences from the same row — one
promises nothing about assertion timing, the other declares the rule undefined —
and *why* they differ is D-030's subject, not this one. What matters here is the
form: a consequence is a statement about this repository, so it cannot rot
against Vishay. A restatement can, and did, nine times at once.

**A reference to what a source does or does not establish points at the record,
not at the source.** Both directions, equally: cite `S-24` rather than
"application note 84323, Revision 06-Mar-2025, page 4, section *Command Code
ALS_IT*", and cite `S-40` rather than restating which sections were read and
found silent. The document coordinates are themselves evidence in both cases, so
they belong in the row with everything else it holds; repeating them elsewhere is
the same defect one level down. A pinned revision changes exactly once in the
record, or it changes in twelve places and eleven of them are wrong until
somebody greps well.

Two exceptions, both narrow and both about provenance rather than device facts.
`docs/vendor/README.md` is the retrieval record and governs source identity, and
the model's README repeats the digests as part of its own source declaration —
a coupling AGENTS.md already documents. `CONTRIBUTING.md` may name a document
when teaching the difference between specified and vendor-stated, because the
example is about the *kind* of source, not about a device fact.

A copy compared by the gate would be a worse version of the same idea: it keeps
the duplication and adds machinery to tolerate it. The status disclosure is
compared that way because two audiences genuinely need the same *disclosure* text
in two packages. A device fact has one home and a name.

**Known limit.** A citation is only as resolvable as the document it names, and
`docs/HARDWARE_CONTRACT.md` does not ship in the package: `crates/veml7700` is
published, and a published crate cannot `include_str!` or package a file outside
its own directory. So a docs.rs reader today sees `S-40` and a consequence
without a way to follow the identifier. That is acceptable and not permanent —
it resolves when the repository becomes public (#6), which is what makes `S-nn`
a citable global name rather than an internal one. It is recorded here so nobody
"fixes" it by copying the contract into the crate.

### What it costs

The registry is a thing to keep true, and D-021 declined a `docs/README.md` on
exactly that reasoning — an index of seven documents is a seventh thing to keep
true. The answer is that this index is machine-checked in both directions, which
the rejected one would not have been. An unchecked index rots; a checked one
fails the build.

Identifiers also make the document less readable in one specific way: allocation
order stops matching document order the first time a row is added. That is
accepted, because the alternative is renumbering, and renumbering is precisely
what breaks every citation elsewhere.
