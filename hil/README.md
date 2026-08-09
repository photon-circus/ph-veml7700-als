# VEML7700 ph-hil project assets

Validate the external mock boundary after installing `ph-hil` separately:

```text
ph-hil shim-build ph-hil-shim.mock.toml --json
ph-hil run hil/plan.toml --bench hil/bench.mock.toml --headless --json
ph-hil analyze project <run> --policy hil/policy.lua
```

The core plan supports digital and relative optical behavior. It does not promote
absolute lux accuracy. `plan-optical.template.toml` remains disabled until the
source, calibrated reference, geometry, window/diffuser, and assessment schema
are frozen. Mock passes are always void for physical support.
