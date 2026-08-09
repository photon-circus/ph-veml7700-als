# GitHub Actions intentionally disabled

This crate is under development and does not use GitHub Actions or a
GitHub-orchestrated self-hosted runner. Active `.yml` and `.yaml` workflow files
are forbidden here by `tools/validate-pack.py`.

Run the complete CI gate directly on a trusted development machine:

```console
./tools/check.sh
```

On Windows PowerShell, run `tools/check.ps1`.

Re-enabling hosted workflow orchestration requires a separate owner-reviewed
repository-policy change.
