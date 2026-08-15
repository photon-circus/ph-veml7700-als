# Verification

> **Authority: contributor procedure and exact conformance inventory.** This
> file records what each test layer can establish and the maintained
> driver-versus-model trace matrix. Consumer-facing consequences stay in the
> crate README and Rustdoc.

## Evidence boundaries

| Layer | Establishes | Does not establish |
| --- | --- | --- |
| Pure/codec | encodings, domains, units, validation | autonomous device behavior |
| Scripted I²C | exact transactions, sequencing, injected failures | an independent device state machine |
| Model-only | the model's declared behavior and unsupported boundaries | agreement with the driver or silicon |
| Driver-versus-model | agreement between independent derivations for named traces | untraced operations, configurations, initial states, or hardware behavior |

The model and conformance packages are repository-only and unpublished. The
driver package has no model dependency, so its standalone tests cannot be
presented as model conformance. Package-archive tests likewise establish only
that the distributable driver builds and passes its own tests.

## Model conformance coverage

Every test named below drives the public driver through the abstract I²C adapter
against the independent model. The gate checks the set of test names in both
directions; review owns the operation, state, and configuration columns. Passing
establishes agreement only within the stated trace.

### Covered

| Public operation | Accepted initial state | Configuration exercised | Conformance test |
| --- | --- | --- | --- |
| `probe` | reset / shut down | — | `probe_accepts_the_fixed_address_little_endian_id` |
| `measure_once` | reset / shut down | ×1/8, 25 ms, cadence disabled | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |
| `measure_once` | active | ×1/8, 25 ms, cadence disabled | `measure_once_from_an_active_start_agrees_with_the_model` |
| `arm_threshold_monitor` | reset / shut down | ×1/8, 100 ms, persistence 1, cadence disabled; programming only | `threshold_monitor_public_operations_program_read_back_and_disable` |
| `arm_threshold_monitor` | reset / shut down | ×1/8, 100 ms, persistence 4, cadence disabled; programming only | `arming_programs_the_field_but_yields_no_modeled_status` |
| `arm_threshold_monitor` | active, monitor disabled | ×2, 100 ms, persistence 4, Mode 2 | `arming_the_monitor_from_an_active_start_agrees_with_the_model` |
| `arm_threshold_monitor` | active, monitor enabled | ×2, 100 ms, persistence 4, Mode 2 | `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
| `read_thresholds` | armed | programming/readback only | `threshold_monitor_public_operations_program_read_back_and_disable`, `arming_programs_the_field_but_yields_no_modeled_status`, `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
| `disable_threshold_monitor` | armed, active | no status-history claim | `threshold_monitor_public_operations_program_read_back_and_disable` |
| `set_measurement_config` | shut down | ×2, 100 ms | `public_power_operations_observe_the_documented_mode_2_refresh_boundary` |
| `set_power_saving` | shut down | ×2, 100 ms, Mode 2 enabled | `public_power_operations_observe_the_documented_mode_2_refresh_boundary` |
| `set_power_state` | reset / shut down or active | requests active only | `measure_once_from_an_active_start_agrees_with_the_model`, `arming_the_monitor_from_an_active_start_agrees_with_the_model`, `public_power_operations_observe_the_documented_mode_2_refresh_boundary`, `public_channel_reads_can_observe_independently_refreshed_generations` |
| `read_als_snapshot` | active | — | `public_power_operations_observe_the_documented_mode_2_refresh_boundary`, `public_channel_reads_can_observe_independently_refreshed_generations` |
| `read_white_snapshot` | active | — | `public_channel_reads_can_observe_independently_refreshed_generations` |
| `read_configuration` | various | — | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state`, `measure_once_from_an_active_start_agrees_with_the_model`, `arming_the_monitor_from_an_active_start_agrees_with_the_model`, `threshold_monitor_public_operations_program_read_back_and_disable`, `arming_programs_the_field_but_yields_no_modeled_status` |
| `read_power_saving` | after restoration | — | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |

### Untraced public operations

| Public operation | Boundary |
| --- | --- |
| `read_device_id` | `probe` exercises the register and codec, but no trace calls this operation. |
| `inspect` | Never called. |
| `snapshot` | Never called; component reads are covered separately. |
| `measure_once_with_timing` | No trace supplies caller-selected timing. |

### Negative boundary trace

`read_threshold_status` is called only to assert the model's exact
`UndefinedQualificationRule` error. No successful status semantics are claimed.

This is stronger than an untraced operation. The model returns an error from
register `0x06` at every configuration — `UndefinedQualificationRule` while
monitoring, `StatusReadWhileMonitorDisabled` otherwise — so the driver's
threshold-status decode and `ThresholdStatusDecodeError` cannot be reached
through any conformance trace at all. That is a structural consequence of
`S-39`, `S-49`, and `S-50`, not coverage awaiting a test. Scripted-transport
tests in the driver crate own that decode path instead.

### Configuration domain

The middle column is deliberately narrow: a value counts as exercised only when
a traced operation *selects* it. A value the model merely powers up in, or that
a restoration assertion reads back, establishes nothing about behavior in that
domain and is recorded separately.

| Domain | Selected by a traced operation | Observed only as reset or restored state | Not exercised or unsupported |
| --- | --- | --- | --- |
| Gain | ×2, ×1/8 | ×1 | ×1/4 |
| Integration time | 25 ms, 100 ms | — | 50, 200, 400, 800 ms |
| Persistence | 1 and 4 programming | — | 2 and 8; qualification at every value is unsupported |
| Power-saving mode | Mode 2 enabled | Mode 1 disabled | Mode 1 enabled, Mode 3, Mode 4 |
| Threshold qualification | none | — | low and high status semantics |

Threshold qualification is an evidence boundary, not a missing test. `S-39`,
`S-49`, and `S-50` do not support a complete oracle, so the model returns
`Unsupported` rather than manufacturing coverage.

Enabled cadence is likewise bounded by evidence rather than by effort. `S-21`
records refresh times only at gain ×2 and `S-22` leaves gain independence
undefined, so the model declines to predict enabled cadence at any other gain.
Traces that pair power saving with a gain now select ×2 for that reason; the
×1/8 plus Mode 2 combination that earlier traces used is no longer
representable, and no trace replaces it. This is a reduction in traced surface
accepted in exchange for not assuming `S-22`.

## Canonical gate

Run `CI_PROFILE=full sh scripts/ci.sh` before a pull request. The script performs
the structural claim checks, formatting, host tests/checks, clippy, Rustdoc,
doctests, five bare-metal builds, dependency policy, package verification, and
tests against the unpacked driver package.

`CI_PROFILE=bounded` is a hosted-feedback subset. It names every skipped step; a
skip is not a pass. `CI_PROFILE=release` adds artifact identity and is reserved
for a separately authorized release workflow.
