# Test plan

## Level 1 — Pure unit tests

- all gain and integration encodings;
- all persistence and power-saving modes;
- reserved configuration/PSM bits rejected;
- reset word `0x0001` decoded;
- expected ID word and byte components;
- all 24 nominal micro-lux scales;
- endpoint multiplication fits `u64`;
- threshold ordering;
- conservative timing math;
- documented PSM cadence and `None` for 25/50 ms.

## Level 2 — Strict transaction tests

Assert exact address, pointer, byte order, transfer length, order, and complete
script consumption for:

- probe (`0x07`, bytes `81 C4`);
- configuration and PSM reads/writes;
- snapshot ALS/white ordering;
- read-modify-write preservation;
- monitor conflict before write;
- threshold arm disable/low/high/PSM/enable ordering;
- every fresh-measurement and restoration stage.

Inject address NACK, bus error, decode error, primary failure, cleanup failure,
and post-capture restoration failure.

## Level 3 — Behavioral model

Model autonomous state rather than static registers:

- shutdown retention;
- explicit shutdown-to-active wake edge before timing;
- rejection of a timing value derived for another integration time before I²C;
- wake-up and integration deadlines;
- ±30 % timing test bounds;
- power-saving refresh cadence;
- independent ALS/white refresh possibility;
- threshold persistence counts;
- monitor status transitions;
- MCU reset while sensor state survives.

## Level 4 — HIL digital and relative optical behavior

- fixed address and ID;
- low-byte-first traffic decoded from logic capture;
- all gain/IT writes and relative count ratios;
- snapshot versus fresh behavior;
- shutdown memorization;
- PSM cadence for documented combinations;
- threshold high/low flags and persistence;
- bus interruption and power-cycle recovery;
- no interrupt-pin assumption.

A repeatable LED source may support relative claims but not calibrated lux.

## Level 5 — Calibrated optical evidence

Required facts:

- fixture revision and geometry;
- sensor package orientation;
- window/diffuser material and transmission;
- source type, spectrum class, drive mode, warm-up, and stability;
- reference instrument identity, range, calibration state, placement, and capture;
- ambient leakage/dark baseline;
- temperature and supply observations;
- saturation and near-dark points.

Sweep representative lux levels across gain/integration combinations. Compare
raw counts and nominal lux without hiding source/window dependence. Mock runs,
compiler success, and uncalibrated LED command values cannot support this level.
