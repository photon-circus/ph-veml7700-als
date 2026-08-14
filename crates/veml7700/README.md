# ph-veml7700-als

Incubating async, allocation-free `no_std` VEML7700 ambient-light driver.

The crate distinguishes register snapshots from deliberately timed fresh
measurements, protects threshold-monitor domains, preserves restoration
failures, and converts ALS counts using integer nominal datasheet scales. It
does not claim calibrated lux or apply application-specific optical correction.

Verification currently consists of pure tests, exact scripted I²C, failure
injection, a coupled test-only autonomous fake, and an independent I²C-level
model covering `probe` and one successful `measure_once` slice. The coupled fake
is not that independent oracle. The repository does not yet contain reviewed
physical-silicon evidence.

The package is not published and retains `publish = false`. See the
[repository README](https://github.com/photon-circus/ph-veml7700-als#readme) and
[driver documentation](https://github.com/photon-circus/ph-veml7700-als/tree/main/docs)
for the complete scope and evidence boundary.

Licensed under MIT.
