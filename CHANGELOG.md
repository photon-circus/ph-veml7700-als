# Changelog

All notable changes to this project will be documented here.

## [Unreleased]

### Added

- Contract-first bootstrap for an async, allocation-free VEML7700 driver.
- Explicit snapshot versus fresh-measurement semantics.
- Integer nominal illuminance scaling for every supported gain/integration pair.
- Threshold-monitor configuration that owns gain, integration time, persistence,
  thresholds, and power-saving cadence as one monitored domain.
- External schema-1 `ph-hil` integration scaffold.
- Repository-enforced Cargo publication guardrails.

### Deferred

- Automatic gain/integration selection.
- Empirical high-illuminance correction.
- Optical-window compensation and source-spectrum calibration.
- A public raw-register interface.
