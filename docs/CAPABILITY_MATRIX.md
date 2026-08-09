# Capability matrix

Status vocabulary: Planned, Mock-integrated, Qualified, Direct, Unsupported.
The bootstrap starts at Planned/Mock-integrated only.

| Capability | Initial status | Promotion evidence |
| --- | --- | --- |
| fixed-address probe and ID | Mock-integrated | physical ID read + logic capture |
| strict configuration codec | Mock-integrated | host tests + physical readback |
| little-endian register words | Mock-integrated | decoded logic capture |
| ALS/white snapshot | Mock-integrated | physical retained/live observations |
| fresh measurement sequencing | Mock-integrated | timing + shutdown-freeze capture |
| nominal integer scaling | Mock-integrated | table tests + source review |
| gain/integration relative behavior | Planned | stable source ratios across settings |
| shutdown memorization | Planned | power/current/data evidence |
| PSM documented cadence | Planned | time-stamped repeated measurement evidence |
| threshold flags and persistence | Planned | controlled crossings + register observations |
| restoration/recovery semantics | Planned | injected interruption and readback |
| calibrated optical accuracy | Planned | calibrated reference and characterized fixture |
| automatic range selection | Unsupported v0.1 | separate API and policy contract |
| empirical high-lux correction | Unsupported v0.1 | application-domain contract and evidence |
| interrupt GPIO | Unsupported | device has no dedicated pin |
