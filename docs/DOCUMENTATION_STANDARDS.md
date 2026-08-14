# Documentation standards

- Use `VEML7700`, `I²C`, `ALS`, `white channel`, `gain ×1/8`, `integration time`,
  `power saving`, `threshold monitor`, and `micro-lux` consistently.
- Never call register `0x06` a physical interrupt pin.
- Every snapshot method states that data may be retained/stale and ALS/white are
  sequential reads.
- Every nominal illuminance method states that it is not calibrated system lux.
- Every complete multi-step operation documents restoration and uncertainty.
- Examples compile as doctests unless explicitly marked `ignore` with rationale.
- Avoid unsupported superlatives such as “accurate lux,” “atomic pair,” or
  “interrupt-driven.”
