# Security policy

## Reporting a vulnerability

Use **Security → Report a vulnerability** on this repository. GitHub private
vulnerability reporting is the preferred route because it keeps the report, the
fix, and the advisory in one place.

It requires a GitHub account. If you would rather not create one, or the report
does not fit that form, email **steve@giacomelli.ca** instead — that route is
equally valid.

Either way, include the affected commit or crate version, the observable effect,
and the smallest reproduction you have.

Do not open a public issue for a flaw that could silently corrupt measurements
or promote nominal or model output as calibrated physical evidence. Ordinary
contract deviations that carry no such risk belong in a normal bug report.

Reports are read and acted on by the maintainer. **No response-time commitment
is offered.** This project has no security team, no on-call rotation, and no
service-level agreement, and it would be dishonest to imply otherwise.

## What is in scope

This repository owns an async, allocation-free `no_std` VEML7700 driver and an
independent behavioral model used only as a test oracle. In-scope reports
concern the code in this repository:

- silent measurement corruption — byte-order, register, scaling, or timing
  defects that yield a plausible but wrong value rather than an error;
- evidence laundering — any path where model, mock, or simulated output can
  reach a caller labeled as hardware-derived or calibrated;
- loss of an invariant the contract promises, such as threshold ordering or
  restoration after a failed one-shot capture;
- dependency or supply-chain problems reachable from this crate's dependency
  graph; and
- defects in the verification gate that would let any of the above pass review.

## What is out of scope

- Physical, electrical, optical, or calibration behavior of the sensor itself.
  This repository establishes no silicon claim; see the status disclosure in the
  README.
- Vulnerabilities in a consuming application, board, or HAL implementation that
  are not caused by this crate.
- Vishay's documents or hardware. Report those to the vendor.

## Supported versions

| Version | Status |
| --- | --- |
| `main` | Best-effort fixes |
| Any published release | None yet — nothing has been published |

The crate is Incubating and unpublished. It has no supported release series, so
there is nothing to backport to. When a release is published, this table will
identify the supported versions explicitly.

## Disclosure

Coordinated disclosure is preferred: report privately, allow a fix to land, then
publish. Because there is no release series, a fix normally reaches consumers as
a commit on `main` rather than as a patch release. If you intend to disclose
publicly on a fixed date, say so in your first message so the timeline is shared
rather than assumed.
