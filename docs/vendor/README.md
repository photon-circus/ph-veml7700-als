# Vendor source record

Do not commit vendor PDFs unless redistribution terms explicitly permit it.
Pin, hash, and review local copies instead.

`docs/vendor/*` is ignored by Git, so vendor documents are never tracked in or
redistributed from this repository. Changing that guardrail requires prior owner
documentation of permissive redistribution rights.

## Required sources

- Vishay VEML7700 datasheet, document 84286, Rev. 1.8, 28-Nov-2024:
  https://www.vishay.com/docs/84286/veml7700.pdf
- Vishay application note “Designing the VEML7700 Into an Application,”
  document 84323, Revision 06-Mar-2025:
  https://www.vishay.com/docs/84323/designingveml7700.pdf
- Product page:
  https://www.vishay.com/en/product/84286/

## Retrieval record

These entries record provenance only. They establish which bytes the repository
interpretation was derived from; they do not check any owner-verification box in
[`HARDWARE_CONTRACT.md`](../HARDWARE_CONTRACT.md).

| Document | Local copy observed | Size | SHA-256 |
| --- | --- | --- | --- |
| 84286 datasheet, Rev. 1.8 | 2026-08-13 | 295,562 bytes | `f338cf7d5911828a2f2ac8ae8324049380c852e34aa5afa43ac92c98ffe827d1` |
| 84323 application note | 2026-08-08 | not recorded | `422f2bea390e145d0d082f40fdeaad4945c79beec159d6600d4007da0aaed558` |

Redistribution permission has not been established for either document, so both
remain untracked.

[`crates/veml7700-model/README.md`](../../crates/veml7700-model/README.md)
repeats these digests as part of the model's source declaration. The two records
are coupled and must change together.

## Owner record to complete

- [x] local copy observed and its SHA-256 recorded;
- [ ] every hardware-contract verification box reviewed;
- [ ] discrepancies between vendor prose and the register tables logged in
      `DECISIONS.md`;
- [ ] any later source revision reviewed before release.
