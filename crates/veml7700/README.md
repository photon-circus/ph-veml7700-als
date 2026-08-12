# ph-veml7700-als

Incubating async, allocation-free `no_std` VEML7700 ambient-light driver.

The crate distinguishes register snapshots from deliberately timed fresh
measurements, protects threshold-monitor domains, preserves restoration
failures, and converts ALS counts using integer nominal datasheet scales. It
does not claim calibrated lux or apply application-specific optical correction.

Verification currently consists of pure tests, exact scripted I²C, failure
injection, and a coupled test-only autonomous fake. The repository does not yet
contain the independent I²C-level mock required for driver cross-validation or
reviewed physical-silicon evidence.

The package is not published and retains `publish = false`. See the
[repository README](https://github.com/photon-circus/ph-veml7700-als#readme) and
[driver documentation](https://github.com/photon-circus/ph-veml7700-als/tree/main/docs)
for the complete scope and evidence boundary.

Licensed under MIT.
