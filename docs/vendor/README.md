# Vendor source record

> **Authority: evidence and provenance record.** This is the single source of
> truth for vendor source identity: URLs, revisions, retrieval facts and
> digests. Where any other document repeats a digest, this file governs.

Do not commit vendor PDFs unless redistribution terms explicitly permit it.
Pin, hash, and review local copies instead.

Three guardrails keep vendor documents out, in increasing order of strength:

- `docs/vendor/*` is ignored by Git, which prevents an accidental `git add`;
- `/docs/vendor/*.pdf export-ignore` in `.gitattributes` keeps any such file out
  of `git archive` output and the GitHub source tarballs;
- `scripts/ci.sh` fails if `git ls-files docs/vendor` reports anything other
  than this README, which is what actually enforces the claim against a
  deliberate `git add -f` or a file tracked before the rule existed.

Only the last is an enforcement check; the ignore rule alone is a default, not a
guarantee. Changing any of the three requires prior owner documentation of
permissive redistribution rights.

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
interpretation was derived from. They do not by themselves establish any device
fact: a matching digest proves the right document was read, not that the
document is right.

The owner has since recomputed both digests over the retrieved local copies and
matched them, which is what closes §1 of
[`HARDWARE_CONTRACT.md`](../HARDWARE_CONTRACT.md). That check is repeatable by
anyone holding the same files.

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
- [x] every hardware-contract verification box reviewed — 44 rows verified,
      including both §1 source-baseline entries. Three rows remain open and each
      says why in place: two are declared **Assumptions** under D-029 that no
      further reading can close, and one is a provisional row still waiting on a
      passage. Reviewed is not the same as all-checked; an open row that states
      its own obstacle is a completed review of that row;
- [x] discrepancies within the vendor documents logged — recorded in
      `docs/HARDWARE_CONTRACT.md` beside the rows they affect rather than in
      `DECISIONS.md`, so a reader meets the caveat with the fact, and each now
      carries a claim identifier so it can be cited. Three so far: `S-46`, a
      stated range of "0 lx to 230 lx" where the table gives 275 lx; `S-47`, a
      ranging example computing 46 lx where its own arithmetic gives 54 lx; and
      `S-45`, *Basic Characteristics* giving `f(SCL)` as a flat 10 kHz to
      400 kHz where *I²C Timing Characteristics* splits standard mode at 100 kHz
      from fast mode at 400 kHz. The first two are prose against a table; the
      third is table against table, and the more specific table governs;
- [ ] any later source revision reviewed before release.
