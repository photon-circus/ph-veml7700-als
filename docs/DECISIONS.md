# Decision log

> **Authority: non-normative rationale.** This records only durable reasons that
> are not evident from code or a governing contract. Device propositions live
> in `HARDWARE_CONTRACT.md`; vendor-source identity lives in
> `vendor/README.md`; current project state does not live here.

## D-012 — Speculative physical infrastructure is out of scope

Unused boards, firmware, runners, and orchestration would create maintenance and
validation obligations before a concrete hardware question exists. A scoped
observation may be recorded when demanded; infrastructure is not prebuilt in
anticipation of one.

## D-015 — Model independence is required

The driver and model share evidence identifiers but no codecs, constants,
timing helpers, state machines, or inference rules. Sharing a derivation would
turn conformance into agreement with itself and remove the model's value as an
independent challenge.

## D-016 — Vendor documents are not redistributed

Redistribution permission has not been established. The repository therefore
records source identity and digests while leaving the vendor files untracked.

## D-017 — One local verification implementation is authoritative

One gate implementation owns verification so hosted and local workflows cannot
silently implement different rules. `full` is authoritative, `bounded` reports
every skip, and `release` adds clean-tree and artifact-identity checks without
taking any publication action.

## D-018 — Visibility and publication remain explicit and independent

Repository visibility, crates.io publication, tagging, and release creation are
separate decisions. None is inferred from readiness, test results, issue
closure, or pull-request approval, and settling one does not settle the next:
that the repository is public and the driver is published authorizes no further
registry action by itself. Each remains maintainer-only.

## D-028 — One authority per subject

The evidence registry, driver contract, model claim, verification record,
release procedure, and changelog each own one subject. Other surfaces provide a
short audience-appropriate summary and link instead of maintaining another
authoritative copy.

## D-030 — Components react independently to undefined behavior

Shared proposition identity does not imply shared policy. The driver may narrow
a promise or choose a defensive policy; the model may require an input, retain
unknown state, or return `Unsupported`. Neither reaction changes the shared
evidence or binds the other component.

## D-031 — Model time is bounded and event-driven

The model processes events in temporal order and rejects an excessive advance.
This keeps execution finite without silently batching transitions whose
equivalence has not been established.

## D-032 — A silence claim needs a located negative

Documentary omission is recorded only after a named, revision-pinned search
scope is checked. It is evidence about the document, not evidence that silicon
does the opposite and not a reason to stop future source review.

## D-033 — Evidence propositions are cited, not restated

An immutable `S-nn` gives the driver, model, conformance tests, hardware
observations, and bug reports one exact referent. Downstream surfaces cite that
ID and state only their own consequence. Copying the proposition, source
coordinates, or artifact creates competing meanings and defeats the graph.

CI enforces only closed structural facts such as identifier uniqueness and
resolution. It does not classify prose, judge evidence strength, infer meaning,
or manufacture follow-up work.

## D-034 — Published artifacts are verified against the commit, not against a local archive

`cargo publish` repackages from the working tree, so a `.crate`'s bytes are a
property of the publishing machine — its platform, its checkout's end-of-line
configuration, its Cargo version — as well as of the commit. Comparing a
downloaded archive against a locally built one therefore fails for correct
releases and cannot distinguish a line-ending difference from edited content.

Verification re-derives each packaged source entry from the Git blobs of the
commit the archive declares, treats an end-of-line-only difference as a pass,
and asserts the declared commit, dependency and feature surface, and locked
versions of the three Cargo-generated entries, which exist in no commit.
`.gitattributes` pins `eol=lf` so future checkouts stop introducing the
difference; the commit-anchored comparison is what makes archives published
before that change verifiable at all.

## D-035 — The verifier may read the registry; the gate may not

Establishing what crates.io holds requires fetching what crates.io holds.
`cargo xtask verify-package --version` performs unauthenticated read-only GETs
of the sparse index and the published archive, and takes no registry action: no
publish, no tag, no release, no credential.

The canonical gate stays offline and verifies only the archive it just built,
so a green gate never depends on network reachability or on registry state. The
accepted cost is a TLS dependency tree in the host tool, four additional
licences in a policy that allowed three, and the advisory surface that comes
with both.
