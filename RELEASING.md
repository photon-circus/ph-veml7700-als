# Releasing

Releases are deliberate maintainer actions. Passing CI or merging preparation
work does not authorize a repository visibility change, crates.io publication,
tagging, or a GitHub Release.

## Status dimensions

Track these independently in the packaged README and crate-level documentation:

- lifecycle;
- distribution state and exact version;
- operation-scoped behavioral-model conformance; and
- physical observation or qualification evidence.

Publication does not imply complete model conformance, physical observation,
hardware qualification, `ph-hil` adoption, or promotion from Incubating.

## 1. Prepare the candidate

Preparation may occur while the repository and crate remain private and
unpublished.

1. Use a lifecycle-matching prerelease version. The first candidate is
   `0.1.0-incubating.1`.
2. Retain `publish = false` until publication is separately approved.
3. Keep the README, crate documentation, changelog, security policy, and model
   declaration consistent with the actual distribution and evidence state.
4. Run `./tools/check.sh` or `./tools/check.ps1` and inspect the generated
   package contents.
5. Review the exact candidate through a pull request.

## 2. Decide repository visibility

After preparation is accepted, a maintainer records whether and when the
repository becomes public. Confirm that its contents are suitable for public
disclosure and that contribution, security, and source links match the chosen
visibility. Do not infer this decision from candidate readiness.

## 3. Decide registry publication

After the visibility decision is recorded and implemented as applicable, a
maintainer separately decides whether to publish the candidate to crates.io.
If publication is approved, open a focused release pull request that:

1. changes `publish = false` to `publish = ["crates-io"]`;
2. changes the distribution disclosure to the exact crates.io prerelease;
3. moves accumulated changes into a dated `0.1.0-incubating.1` changelog
   section while preserving an `Unreleased` section;
4. runs the complete canonical gate and inspects the exact package;
5. runs `cargo publish --dry-run`; and
6. records the verified commit and artifact for maintainer review.

Registry credentials and `cargo publish` remain outside ordinary CI.

## 4. Publish the reviewed artifact

Only after the release pull request is approved and merged:

1. verify the release commit and rerun the documented release gate;
2. tag that commit as `v0.1.0-incubating.1`;
3. publish the already reviewed package from that commit;
4. create a GitHub Release from the same tag using the matching changelog
   section and mark it as a prerelease; and
5. verify the crates.io and GitHub records before announcing availability.

Crates.io versions are permanent. If publication fails after any durable
release record is created, do not move or reuse that record; document the
failure and prepare a higher version when required.
