# Implementation plan

## M0 — Contract and repository bootstrap

- freeze source revisions and record hashes;
- validate workspace, lints, local CI runners, agent instructions, and
  documentation set;
- hard-disable Cargo publication and reject publish-enabled manifests in pack
  validation;
- run pack validator.

Exit: contracts reviewed enough to implement without guessing.

## M1 — Pure codecs

- register pointers and little-endian word codec;
- gain, integration, persistence, shutdown, monitor-enable encoding;
- strict reserved-bit decoding;
- power-saving codec and documented cadence table;
- ID decoding;
- nominal micro-lux table.

Exit: exhaustive host tests for every legal encoding and reserved value class.

## M2 — Read-only facade

- inert construction and release;
- probe and ID classification;
- configuration, PSM, thresholds, status, ALS, white reads;
- inspect and snapshot semantics;
- contextual bus errors.

Exit: exact scripted-I²C transactions, including low-byte-first words.

## M3 — Controlled state changes

- measurement-config setter;
- shutdown/active setter;
- power-saving setter;
- monitor-domain conflict rejection.

Exit: read-modify-write tests preserve unrelated fields and reject retargeting
before any write.

## M4 — Complete fresh measurement

- conservative timing policy;
- disable PSM, activate requested domain, wait, freeze, read, restore;
- staged primary, recovery, and post-capture restoration failures;
- fresh provenance and nominal scaling.

Exit: transaction-order and injected-failure tests for every stage.

## M5 — Threshold monitor

- complete semantic config;
- disable-first, threshold writes, PSM, enable-last;
- status polling and no-GPIO documentation;
- autonomous fake model with persistence and cadence.

Exit: monitor cannot be silently retargeted and enable-last is exact.

## M6 — External HIL mock integration

- capabilities, plan, contract, transcript, Lua modules, policy, build hooks;
- deterministic mock run and policy assessment;
- mock evidence remains void.

Exit: schema alignment passes in the local CI runner.

## M7 — Physical digital/relative validation

- real managed harness;
- logic-analyzer proof of wire order and register sequences;
- switchable optical levels for relative gain/IT, shutdown retention, PSM cadence,
  threshold persistence, and recovery;
- sealed run review.

## M8 — Calibrated optical validation

- characterized enclosure/window/diffuser and stable source;
- calibrated reference luxmeter or photometric instrument;
- multi-level and multi-source sweeps;
- uncertainty budget and policy assessment.

Only M8 can promote absolute optical capability. Empirical correction remains a
separate future milestone.
