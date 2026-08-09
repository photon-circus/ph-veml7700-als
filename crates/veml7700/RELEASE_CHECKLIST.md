# Release checklist

- [ ] Hardware contract signed against pinned Vishay documents.
- [ ] Public API contract frozen and all public items represented.
- [ ] `cargo fmt`, tests, all-feature check, Clippy, rustdoc, doctests, and cross targets pass.
- [ ] `cargo deny` and package construction/listing pass.
- [ ] `publish = false` remains enforced and automation has no registry credentials.
- [ ] Mock HIL integration passes without being described as hardware validation.
- [ ] Sealed physical runs support every promoted capability.
- [ ] Optical claims identify fixture, window/diffuser, source spectrum, reference meter, and calibration state.
- [ ] Changelog, version, documentation links, and evidence references are current.
