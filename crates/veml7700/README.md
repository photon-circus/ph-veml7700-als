# ph-veml7700-als

Incubating async, allocation-free `no_std` VEML7700 ambient-light driver.

> [!WARNING]
> **Lifecycle:** Incubating.
> **Distribution:** Unpublished; the candidate version is
> `0.1.0-incubating.1` and the manifest retains `publish = false`.
> **Model conformance:** An independent I²C-level model covers `probe` and one
> successful `measure_once` path only. All other public operations are outside
> the current model claim.
> **Physical evidence:** None. Evidence applies only to the named operations,
> and eventual publication would not imply hardware qualification.

The crate distinguishes register snapshots from deliberately timed fresh
measurements, protects threshold-monitor domains, preserves restoration
failures, and converts ALS counts using integer nominal datasheet scales. It
does not claim calibrated lux or apply application-specific optical correction.

Driver verification currently consists of pure codec tests, exact scripted I²C
with failure injection, and the bounded independent model described above. The
repository also carries a coupled autonomous-state fake, but its tests exercise
that fake directly rather than the driver, so it contributes no driver
evidence.

The package is not published and retains `publish = false`. See the
[repository README](https://github.com/photon-circus/ph-veml7700-als#readme) and
[driver documentation](https://github.com/photon-circus/ph-veml7700-als/tree/main/docs)
for the complete scope and evidence boundary.

Licensed under MIT.
