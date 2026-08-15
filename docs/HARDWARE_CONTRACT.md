# VEML7700 hardware contract

> **Authority: normative.** Interpreted device behavior. Every device claim in
> this repository derives from here, and a row is only as strong as its
> verification state.

Binding interpreted device facts for `ph-veml7700-als`. A checked row means the
owner has verified the recorded official Vishay source. Unchecked rows remain
provisional and must not be promoted to physical-support claims.

Verification is tracked per fact rather than per section, because a section can
be partly source-backed: §6 and §8 each have both.

A row is in one of three states, and they must not be conflated:

| State | Meaning |
| --- | --- |
| `[x]` | Reviewed against the pinned sources and confirmed. |
| `[ ]` | **Provisional — not yet reviewed.** No claim either way. |
| `[ ]` with an explicit note | Reviewed, and the sources do not state the fact. The absence is the finding. |
| `[ ]` marked **Assumption** | Reviewed, unstated, **and unresolvable by further reading.** The driver relies on it, so it is declared rather than left implicit. Closing it needs physical evidence. |

Only the third form records a confirmed omission, and it says so in the row. An
unchecked row with no note means nobody has looked yet; it is not evidence that
the sources are silent.

The fourth exists because "not in the sources" and "not knowable from the
sources" are different problems. A missing statement might be found on another
page. An assumption about how silicon behaves cannot be — no amount of reading
resolves it, and the honest response is to name it, name what it would take to
settle, and let the driver's behavior depend on it visibly.

Each Assumption row therefore names an **observation**, in one sentence. It does
not carry a procedure: this repository excludes physical-evidence plans, so the
procedures live in #58 and the rows link to it.

Counts of "verified" rows in the changelog and elsewhere refer to the bullet
rows in §2 onward. The two §1 source-baseline entries are tracked in that
section's table and are verified; every other row depends on them.

A row that fails to verify becomes its own issue rather than an in-place edit,
because correcting the contract is a behavior change to the driver, the model,
or both. See #21.

## 1. Source baseline

| Source | Revision | Status |
| --- | --- | --- |
| VEML7700 datasheet, document 84286 | Rev. 1.8, 28-Nov-2024 | [x] owner verified |
| Designing the VEML7700 Into an Application, document 84323 | 06-Mar-2025 | [x] owner verified |

Both rows were closed by the owner computing SHA-256 over the retrieved local
copies and matching the digests recorded in
[`docs/vendor/README.md`](vendor/README.md) and the model's source declaration.
The two records agree, so every row below is anchored to a known byte sequence
rather than to "the datasheet" as a moving target — a later Vishay revision
becomes a visible digest change and a re-verification, not a silent drift.

What this establishes is provenance and nothing further. It fixes *which bytes*
were read; it says nothing about whether those bytes describe the silicon
correctly. A verified row means the repository recorded the source faithfully.
Only physical evidence can make it a claim about a part.

The datasheet and application note occasionally use inconsistent prose such as
“six registers” while their command table includes `0x00` through `0x07` and a
power-saving register at `0x03`. The explicit command/register tables and ID
section govern this driver; the discrepancy remains documented rather than
silently normalized.

## 2. Electrical and bus boundary

- [x] Sensor supply `V_DD` operating range is 2.5 V to 3.6 V. The Product
      Summary names this `OPERATING VOLTAGE RANGE`; Basic Characteristics gives
      `V_DD` as MIN 2.5 V, TYP 3.3 V, MAX 3.6 V.
- [x] **`I²C BUS VOLTAGE RANGE` is 1.7 V to 3.6 V.** Datasheet 84286 Rev. 1.8,
      page 1, `PRODUCT SUMMARY`, where it is a column of its own beside
      `OPERATING VOLTAGE RANGE`. The same 1.7 V to 3.6 V rail is labelled in the
      datasheet's own application circuit (*Application*, section *2. I²C
      Interface*), and application note 84323 Rev. 06-Mar-2025, page 2,
      *Application Circuitry for the VEML7700*, states it in prose: the sensor
      connects to a 2.5 V to 3.6 V supply, and "the pull-up resistors at the I²C
      bus lines may also be connected to a power supply between 1.7 V to 3.6 V".

      Recorded under the source's own parameter name and nothing further.

      **Located negative (D-032)** for what this row declines to supply. Read:
      datasheet 84286 Rev. 1.8 — `PRODUCT SUMMARY`, *Basic Characteristics*,
      *I²C Timing Characteristics* and its Note (1), and *Application* section
      *2. I²C Interface* with its circuit figure; application note 84323
      Rev. 06-Mar-2025 — *Application Circuitry for the VEML7700*. In those
      sections:

      - level shifting is not discussed at all, and the term does not appear;
      - the timing table is indexed by standard and fast mode only, with no
        voltage term, and Basic Characteristics gives `f(SCL)` with no voltage
        condition, so neither states a minimum bus voltage for any clock rate;
      - `V_ih` is given under the condition `V_DD` = 3.3 V, which is the sensor
        supply and not this rail, and no passage in the sections above relates
        the two.

      Absence outside those sections is not claimed.
- [x] I²C bus **input** H-level range `V_ih` is 1.3 V to 3.6 V, and input L-level
      range `V_il` is −0.3 V to 0.4 V, both specified at `V_DD` = 3.3 V (Basic
      Characteristics). These are signal thresholds on `SCL`/`SDA`, **not** a
      supply. The datasheet's I²C Interface section restates the first as
      "I²C H-level range = 1.3 V to 3.6 V".

      **Three distinctly named quantities meet here and must not be merged:** the
      sensor supply (2.5 V to 3.6 V), the bus rail the pull-ups may use (1.7 V to
      3.6 V), and the input thresholds (1.3 V to 3.6 V, −0.3 V to 0.4 V, both at
      `V_DD` = 3.3 V).

      History, because the row has been wrong in both directions. It once
      recorded a "high-level supply … 1.7 V to 3.6 V", merging the second and
      third. #54 correctly separated them and then overcorrected, concluding that
      1.7 V appears nowhere in the source; it appears in the three places listed
      in the row above. #54 had itself enumerated "1.7 V is from a different
      parameter" as a possible outcome — that was the right one, and it was
      passed over because the search stopped at the passage the dispute quoted.
      See #67 and D-032.
- [x] Clock frequency `f(SMBCLK)` is 10 kHz to 100 kHz in standard mode and
      10 kHz to 400 kHz in fast mode (*I²C Timing Characteristics*). The two
      modes have different maxima; a single 10–400 kHz range would wrongly
      permit standard mode at 400 kHz. The source marks these values as based on
      the standard I²C protocol requirement and **not tested in production**.

      One table-versus-table discrepancy is recorded rather than normalized:
      *Basic Characteristics* separately gives `f(SCL)` as a flat 10 kHz to
      400 kHz with no mode split, which is the value that would permit standard
      mode at 400 kHz. The dedicated timing table governs, being the more
      specific statement of the same quantity — consistent with §1, where the
      explicit tables govern loose prose. Nothing in this driver selects or
      constrains a bus mode; the bus is the caller's.
- [x] The fixed 7-bit address is `0x10` (`0x20` write / `0x21` read in 8-bit form).
- [x] Pull-ups are external; the vendor suggests values above 1 kΩ, commonly
      2.2 kΩ to 4.7 kΩ.
- [x] The driver never owns sensor power, board pull-ups, cover-window geometry,
      or an external optical source.

## 3. Word transfer order

Every command register is 16 bits. The I²C protocol transfers **data byte low,
then data byte high** for both writes and reads.

Consequences:

- `u16::from_le_bytes([low, high])` is mandatory on reads;
- `value.to_le_bytes()` is mandatory on writes;
- a big-endian register helper is a review-blocking defect;
- strict transaction tests inspect exact byte order.

- [x] Low byte then high byte for every 16-bit register, on reads and writes.
      Fig. 9 shows both frames explicitly: a write sends command code, then
      `Data byte (LSB)`, then `Data byte (MSB)`; a read returns them in the
      same order after the repeated start. This is the row a big-endian helper
      would violate, and it is now source-backed rather than inferred.

## 4. Register map

| Pointer | Semantic name | Access | Driver treatment | Reset value |
| --- | --- | --- | --- | --- |
| `0x00` | configuration | R/W | strict typed codec | `0x0001` |
| `0x01` | high threshold | R/W | raw ALS counts | not declared by sources |
| `0x02` | low threshold | R/W | raw ALS counts | not declared by sources |
| `0x03` | power saving | R/W | strict typed codec | not declared by sources; **assumed** `0x0000` (below) |
| `0x04` | ALS output | R | snapshot/fresh result | not declared by sources |
| `0x05` | white output | R | raw spectral companion channel | not declared by sources |
| `0x06` | threshold status | R | bit 15 low, bit 14 high | not declared by sources |
| `0x07` | ID | R | expected word `0xC481` at the fixed address option | source-declared identity `0xC481` |

No public raw-register accessor exists in v0.1.

- [x] Pointer values and access direction for all eight registers, from the
      COMMAND REGISTER FORMAT table: `00` ALS_CONF_0 R/W, `01` ALS_WH R/W,
      `02` ALS_WL R/W, `03` Power saving R/W, `04` ALS R, `05` WHITE R,
      `06` ALS_INT R, `07` ID R. This is the table §1 names as governing, and
      it resolves the prose that calls `03h` "not defined": the register format
      defines it as power saving.
- [x] Which registers have a source-declared reset value, and which do not.
      `0x00` is declared (`0x0001`, register-format note) and `0x07` carries a
      declared fixed identity (Table 8). No other register has a declared reset
      **word**. `0x03` is the partial case: Table 4 constrains bits 15:3 to zero,
      which is a write validity rule rather than a power-on value, while the
      application note's power-saving discussion says the default is mode 1 =
      `00` for bits 2:1 — a statement about the `PSM` field only, leaving
      `PSM_EN` and therefore the word undeclared. See the Assumption row below
      and #71.

      **Located negative (D-032)** for the "not declared by sources" cells in the
      table above. Read: datasheet 84286 Rev. 1.8 — the COMMAND REGISTER FORMAT
      overview, the per-register Command Code #0 through #7 sections, and
      Tables 1, 4, 7 and 8; application note 84323 Rev. 06-Mar-2025 — every
      COMMAND REGISTER FORMAT block it repeats. Across both documents the word
      *reset* does not appear in any register context, and exactly **two**
      default statements exist: the register-format note for command code 0, and
      the application note's `PSM` mode-1 sentence recorded above. Neither
      document gives a power-on value for `0x01`, `0x02`, `0x04`, `0x05`, or
      `0x06`.

      Absence outside those sections is not claimed. What is claimed is that the
      places a reset value would be stated — a register's own definition — were
      read and do not state one.

- [ ] **Assumption: register `0x03` reads `0x0000` before it is written.**
      *Requires physical validation.*

      **Located negative (D-032).** Read: datasheet 84286 Rev. 1.8, the
      command-register overview table, Table 4 *Power Saving Modes*, and the
      register-format note that declares a power-on default for command code 0;
      application note 84323 Rev. 06-Mar-2025, the *Command Code PSM / PSM_EN*
      register-format table and the power-saving discussion preceding it. The
      datasheet declares a power-on default exactly once, for `0x00`, and gives
      `0x03` only `Set (15 : 3) 0000 0000 0000 0b` — a write validity rule for
      reserved bits, not a power-on value. Table 4 gives field encodings with no
      reset column.

      **One passage bears on part of it.** The application note, describing how
      to enable power saving, says the default it comes up with is mode 1 = `00`
      for bits 2 and 1. That speaks to the `PSM` field reading `00` before it is
      written. It says nothing about `PSM_EN`; that bit's power-on state is only
      implied by the instruction to set it to 1 in order to activate the feature,
      and an implication is not a declaration.

      So the undeclared part is `PSM_EN`, and with it the full word. `0x0000` is
      the value every defined field takes at zero — `PSM_EN` disabled, `PSM`
      mode 1, reserved bits clear — so a device that has never had power saving
      written is expected to read it. Narrowing this row to `PSM_EN` alone is a
      contract-state change and belongs to #71, not to an in-place edit here.

      **The driver does not rely on this.** Every path reads register `0x03`
      before acting on it, so a different power-on value changes nothing the
      driver does. The dependency is confined to the model, whose construction
      uses `RESET_POWER_SAVING` to represent a device in its power-on state, and
      it is declared as an abstraction in the model README rather than presented
      as derived behavior.

      **What would settle it:** reading `0x03` on a device that has not been
      written since power-on. See #58.

## 5. Configuration register `0x00`

Reset/default word is `0x0001`, meaning shutdown with gain ×1, 100 ms,
persistence one, and threshold monitoring disabled.

| Bits | Field | Encodings |
| --- | --- | --- |
| 15:13 | reserved | must be zero |
| 12:11 | gain | `00` ×1, `01` ×2, `10` ×1/8, `11` ×1/4 |
| 10 | reserved | zero |
| 9:6 | integration | `1100` 25 ms, `1000` 50 ms, `0000` 100 ms, `0001` 200 ms, `0010` 400 ms, `0011` 800 ms |
| 5:4 | persistence | 1, 2, 4, 8 qualifying measurements |
| 3:2 | reserved | zero |
| 1 | threshold monitor enable | disabled/enabled |
| 0 | shutdown | 0 active, 1 shutdown |

Reserved integration encodings and non-zero reserved bits are decode errors, not
values to preserve as ordinary typed state.

- [x] Reset/default word `0x0001`. The COMMAND REGISTER FORMAT note states it
      directly: *command code 0 default value is 01 = devices is shut down*.
      With every other field zero that is gain ×1, 100 ms, persistence 1 and the
      monitor disabled — which is what this contract records. **Table 1 does not
      establish this.** It gives bit meanings only and states no power-on
      value. `0x0001` is consistent with every field zero except `ALS_SD`, but
      consistency is not a source: the claim is about what the device powers up
      in, and that needs a passage that says so.

      **Located negative (D-032)** for that sentence. Read: datasheet 84286
      Rev. 1.8, Table 1 *ALS_CONF_0 #0* in full — every row is a bit range, a
      field name, and its encodings, with no reset or default column — and the
      Command Code #0 section around it. The default is stated in the adjacent
      register-format note, not in Table 1, which is the whole point of
      separating them here: a reader who cites "Table 1" for the reset word is
      citing the wrong object.
- [x] Reserved bits 15:13, stated by the source as `000b`.
- [x] Gain encodings: `00` ×1, `01` ×2, `10` ×1/8, `11` ×1/4. Note that the
      encoding order is not the magnitude order — `10` is ×1/8 and `11` is ×1/4,
      so a table sorted by gain does not match a table sorted by bit pattern.
- [x] Integration-time encodings, bits 9:6: `1100` 25 ms, `1000` 50 ms, `0000`
      100 ms, `0001` 200 ms, `0010` 400 ms, `0011` 800 ms. Like gain, the
      encoding order is not the magnitude order.
- [x] Persistence encodings, bits 5:4: `00` 1, `01` 2, `10` 4, `11` 8
      (Table 1, `ALS_PERS`).
- [x] Monitor-enable, bit 1 (`ALS_INT_EN`: 0 disable, 1 enable), and shutdown,
      bit 0 (`ALS_SD`: 0 power on, 1 shut down), from Table 1.
- [x] Reserved bits: 15:13 `000b`, bit 10 `0b`, and 3:2 `00b` (Table 1).
- [x] **Reconfiguration requires shutdown first.** The source's own software
      flow sets `ALS_SD = 1` (standby) before any reconfiguration, changes gain
      or integration time while shut down, and clears `ALS_SD` afterwards.

### Reconfiguration sequence

This is a positive source requirement, not merely an absence of permission to
write while active. The vendor's flow chart annotates the step directly: *before
any reconfiguration set `ALS_SD` to 1 = stand_by*.

Two consequences follow, and #29 owns both:

1. The independent model's rejection of changed or repeated active configuration
   is **correct and source-backed**. It should not be relaxed to admit the
   driver's current behavior.
2. `set_measurement_config`, and every other path that writes configuration or
   power-saving fields, must enter shutdown first, write while shut down, and
   restore the active state last. The driver currently reads the configuration
   and writes the new one without entering shutdown, so it can reconfigure an
   active sensor. That is the mismatch #29 records, and this row resolves it
   against the driver rather than against the model.

Correcting it is a behavior change with its own tests and contract updates, so
it belongs to #29 and not to this verification pass.

## 6. Power-saving register `0x03`

- bits 15:3 are reserved and must be zero;
- bits 2:1 select Mode 1 through Mode 4;
- bit 0 enables power-saving cadence.

The vendor explicitly documents refresh times for 100, 200, 400, and 800 ms:

| Mode | 100 ms | 200 ms | 400 ms | 800 ms |
| --- | ---: | ---: | ---: | ---: |
| 1 | 600 ms | 700 ms | 900 ms | 1300 ms |
| 2 | 1100 ms | 1200 ms | 1400 ms | 1800 ms |
| 3 | 2100 ms | 2200 ms | 2400 ms | 2800 ms |
| 4 | 4100 ms | 4200 ms | 4400 ms | 4800 ms |

The driver does not extrapolate a documented refresh interval for 25 or 50 ms.
The source table has no rows for those integration times, which is why they are
unsupported rather than computed.

**Located negative (D-032)** for that absence. Read: the *Refresh Time, I_DD,
and Resolution Relation* table in datasheet 84286 Rev. 1.8 and its identical
copy in application note 84323 Rev. 06-Mar-2025, plus the app note's separate
`PSM` / `ALS_IT` → refresh-time table. All three are indexed by `ALS_IT` ∈
{100, 200, 400, 800} ms; neither 25 ms nor 50 ms appears as a row in any of
them, in either document.

The source records this relation at ALS gain ×2 only. The driver treats refresh
time as independent of gain — `nominal_refresh_time_ms` takes an integration
time and no gain. The pattern is exact (refresh = integration + 500, 1000, 2000,
or 4000 ms for Modes 1 to 4), but exactness is not the same as a source
statement, so the inference is recorded here rather than left implicit in code.

- [x] Register `0x03` field layout (Table 4): bits 15:3 reserved, bits 2:1
      `PSM` selecting `00` mode 1 through `11` mode 4, bit 0 `PSM_EN`
      (0 disable, 1 enable).
- [x] The sixteen refresh times above match the vendor's refresh time / I_DD /
      resolution relation, at gain ×2.
- [ ] **Assumption: refresh time is independent of ALS gain.**
      *Requires physical validation. Further reading cannot close this row.*

      **Located negative (D-032).** Read: datasheet 84286 Rev. 1.8, section
      *Refresh Time Determination of PSM* and the *Refresh Time, I_DD, and
      Resolution Relation* table; application note 84323 Rev. 06-Mar-2025, the
      power-saving discussion at *Command Code PSM / PSM_EN* and its own copy of
      the *Refresh Time, I_DD, and Resolution Relation* table. Every one of the
      sixteen rows in that table, in **both** documents, is `ALS_GAIN = x2`; no
      other gain appears anywhere in it. The app note's separate
      `PSM` / `ALS_IT` → refresh-time table is indexed without a gain term at
      all. Neither document states a gain dependence, and neither states
      independence.

      An unindexed table is not a claim of independence. The relation carries no
      gain term (`integration + 500, 1000, 2000 or 4000 ms`), and independence is
      the common understanding in third-party libraries, but neither is a source
      statement and this row does not accept one as a substitute.

      **This driver assumes it.** `nominal_refresh_time_ms` takes an integration
      time and no gain, so every cadence figure it returns is gain-independent by
      construction. The independent model inherits the same assumption through
      `refresh_interval_ns`.

      **What would settle it:** measuring refresh interval at a fixed integration
      time and power-saving mode across all four gains, on silicon. If any
      differs, both the driver signature and the model change. Nothing in the
      documents can substitute for that observation, which is why this is
      declared rather than left open. See #58.

## 7. Wake-up, integration, and freshness

- after changing shutdown bit from 1 to 0, wait at least 2.5 ms before the first
  measurement;
- the vendor states that a ±30 % integration-time tolerance can be assumed, and
  that it should be considered when reading measurement results;
- data registers retain the last ambient result during shutdown;
- waking causes later refresh by a new detection;
- a plain register read therefore cannot prove freshness.

The complete fresh operation waits:

```text
2.5 ms wake-up + 130 % of selected integration time + software margin
```

It then enters shutdown before reading ALS and white so that autonomous refresh
cannot occur between those two sequential register reads. This is a software
coherence policy, not a vendor-stated atomic pair primitive.

- [x] The 2.5 ms minimum wake-up delay after clearing the shutdown bit. The
      source's flow chart states `ALS_SD = 0`, then wait ≥ 2.5 ms.
- [x] **The vendor states that a ±30 % integration-time tolerance can be
      assumed, and that it should be considered when reading measurement
      results.** Application note 84323, Revision 06-Mar-2025, page 4, section
      *Command Code ALS_IT*, `Remark`:

      > For the integration time a tolerance of ± 30 % can be assumed. This
      > tolerance should also be considered during the read out of the
      > measurement results.

      **This is application guidance, not a characterized guarantee**, and the
      two must not be conflated. The figure appears in the application note
      only. The datasheet gives no integration-time tolerance in Absolute
      Maximum Ratings or Basic Characteristics, and publishes no oscillator
      accuracy for this part — the oscillator appears in the block diagram and
      nowhere in the electrical tables. The vendor's own wording is *can be
      assumed*, which is a design allowance, not a specified min/max.

      So this row records that ±30 % is **vendor-stated**. It does not record a
      worst case guaranteed across process, voltage, and temperature. Integration
      intervals are counted off the part's internal oscillator, and nothing in
      either source bounds that oscillator's spread over PVT.

      **This driver applies it.** `INTEGRATION_TOLERANCE_PERCENT` is why the
      conservative wait is 130 % of the selected integration time, which is this
      driver acting on the Remark's second sentence. The margin is therefore
      conservative *given the vendor's stated tolerance* rather than in general:
      a real spread wider than ±30 % would make the driver read a register before
      the conversion behind it completed — the freshness guarantee fails
      silently, returning a stale value that is indistinguishable from a new one.

      **What measurement would add:** clocking actual conversion completion
      against the shutdown-to-active wake edge would turn vendor guidance into
      characterization evidence for the parts measured. That is optional
      compliance work, not the discovery of a missing figure, and no row depends
      on it. Sampling considerations are in #58, not here — this document does
      not carry a physical-evidence plan.
- [x] Data registers retain the last result while shut down. The source calls
      this *Auto-Memorization*: the part memorizes the last ambient data before
      shutdown, the host may read it directly while shut down, and on wake the
      data is refreshed by a new detection. That last clause is also why a
      plain register read cannot prove freshness.

## 8. ALS and white channels

Both outputs are unsigned 16-bit counts. The ALS channel follows photopic
response and has a nominal gain/integration-dependent lux scale. The white
channel has broader spectral response and is returned as counts only in v0.1.

The nominal resolution table is:

| IT | gain ×2 | gain ×1 | gain ×1/4 | gain ×1/8 | unit |
| ---: | ---: | ---: | ---: | ---: | --- |
| 800 ms | 0.0042 | 0.0084 | 0.0336 | 0.0672 | lx/count |
| 400 ms | 0.0084 | 0.0168 | 0.0672 | 0.1344 | lx/count |
| 200 ms | 0.0168 | 0.0336 | 0.1344 | 0.2688 | lx/count |
| 100 ms | 0.0336 | 0.0672 | 0.2688 | 0.5376 | lx/count |
| 50 ms | 0.0672 | 0.1344 | 0.5376 | 1.0752 | lx/count |
| 25 ms | 0.1344 | 0.2688 | 1.0752 | 2.1504 | lx/count |

The source states the maximum possible illumination for each pair. It is the
resolution multiplied by 65 535, and all twenty-four entries are internally
consistent with the resolution table:

| IT | gain ×2 | gain ×1 | gain ×1/4 | gain ×1/8 | unit |
| ---: | ---: | ---: | ---: | ---: | --- |
| 800 ms | 275 | 550 | 2 202 | 4 404 | lx |
| 400 ms | 550 | 1 101 | 4 404 | 8 808 | lx |
| 200 ms | 1 101 | 2 202 | 8 808 | 17 616 | lx |
| 100 ms | 2 202 | 4 404 | 17 616 | 35 232 | lx |
| 50 ms | 4 404 | 8 808 | 35 232 | 70 463 | lx |
| 25 ms | 8 808 | 17 616 | 70 463 | 140 926 | lx |

The core uses exact integer micro-lux-per-count values.

### Linearity and the correction polynomial

The source constrains where uncorrected counts mean anything:

- above about 100 lx, gain ×1 and ×2 should not be used because the sensor
  becomes non-linear;
- when using gain ×1/4 or ×1/8, the correction formula should be used; and
- above 1 000 lx a correction formula needs to be applied.

The vendor's polynomial, applied to the uncorrected lux value, is:

```text
corrected = a·x⁴ + b·x³ + c·x² + d·x
a = 6.0135e-13   b = -9.3924e-9   c = 8.1488e-5   d = 1.0023
```

**This driver does not apply it.** That remains D-007, and the reason is
unchanged: applicability depends on optical window, source spectrum, geometry,
and application validation, none of which a driver owns.

The consequence must be stated plainly rather than left implied. The driver's
own low-gain presets sit inside the range where the source says correction is
needed, so `nominal_illuminance` above roughly 1 000 lx is an uncorrected value
the vendor does not consider a lux estimate. It is honest as *nominal* output and
is never calibrated system lux.

The coefficients are recorded here as a device fact, not as work owed by this
crate. Evaluating a quartic on the target would also mean floating point, which
this driver does not use anywhere: both crates are integer-only, and several
supported triples have no FPU.

The intended home is [`ph-curves`](https://github.com/photon-circus/ph-curves),
whose transfer functions fit a curve **host-side** and emit integer or
fixed-point tables, so firmware evaluates without floating point. It is
explicitly not a driver crate and does not touch buses or device lifecycle, so
the boundary matches D-007 from the other side. `ph-temt6000-als` already pairs
an illuminance integration layer with it; a corrected-lux layer for this part
would follow that shape rather than move correction into the driver.

Nothing above commits this repository to building that layer. It records where
the work belongs if it is done.

- [x] The complete twenty-four-entry resolution table, every gain and every
      integration time.
- [x] The twenty-four-entry maximum-detection-range table.
- [x] Gain ×1/8 at 25 ms is 2.1504 lx/count, reaching 140 926 lx — the widest
      range the part offers. The Basic Characteristics table names that exact
      pair as the *detectable maximum illuminance* condition, `E_V max` =
      140 000 lx, so the widest-range preset is the configuration the source
      itself uses to state the part's maximum. It also gives 0.0042 lx/step at
      ×2 and 800 ms, matching the opposite corner of the resolution table.
- [x] Above 100 lx, gain ×1 and ×2 are outside the linear region.
- [x] Correction is called for with gain ×1/4 and ×1/8, and above 1 000 lx.
- [x] The correction polynomial coefficients, checked against the source's own
      worked example: 5581 counts at ×1/4 and 100 ms give 1500 lx uncorrected
      and 1658 lx corrected.
- [x] ALS output is a 16-bit word.
- [x] The white channel is a 16-bit count: command code `05` is defined as
      *MSB 8 bits data of whole WHITE 16 bits* and *LSB 8 bits*, read-only, with
      all sixteen bits carrying data. No sign bit is defined for it or for ALS,
      and no passage describes either as signed.

      **Located negative (D-032)** for that last sentence. Read: datasheet 84286
      Rev. 1.8 — the COMMAND REGISTER FORMAT entries for `04` (ALS) and `05`
      (WHITE), the Command Code #4 and #5 sections, and Basic Characteristics;
      application note 84323 Rev. 06-Mar-2025 — *Read-Out of ALS Measurement
      Results*, *Transferring ALS Measurement Results into a Decimal Value*, and
      its COMMAND REGISTER FORMAT blocks for the same two registers. Both
      registers are described only as MSB and LSB halves of a whole 16-bit
      value; neither document uses *sign*, *signed*, or *two's complement* of
      either channel anywhere. The app note's own worked example decodes a raw
      word as an unsigned decimal count.

### Starting configuration and ranging

The source gives explicit application guidance, and it decides what a first-use
preset should be:

- for unknown brightness the application should always start at the lowest gain,
  ×1/8 or ×1/4, to avoid overload if strong sunlight suddenly reaches the sensor;
- to show such a high value, an integration time **lower than 100 ms may be
  needed**;
- gain ×1 and ×2 are for low illumination below 100 lx — at 100 ms they saturate
  at 4 404 lx and 2 202 lx respectively; and
- linear behavior runs from 0.0042 lx to about 1 klx.

The source also sketches an auto-ranging loop — start at ×1/8, and while counts
stay at or below 100, step the gain up through ×1/4, ×1, ×2, then lengthen the
integration time toward 800 ms. It places that loop in **application software**,
which is where this driver leaves it. Automatic range selection stays a non-claim
under §11, and this is the source's own framing rather than a driver limitation.

- [x] Start at the lowest gain for unknown brightness, and reduce integration
      time below 100 ms to cover the brightest conditions.
- [x] Gain ×1 and ×2 are confined to illumination below 100 lx.
- [x] Linear behavior spans 0.0042 lx to about 1 klx.
- [x] Auto-ranging is application-software responsibility in the source.

Two prose-versus-table discrepancies in this section are recorded rather than
normalized:

1. The narrative calls 0.0042 lx/count a range of "approximately 0 lx to 230 lx",
   while the table gives 275 lx for that pair.
2. The ranging example states that 100 counts at ×1/8 is 54 lx, then that after
   switching to ×1/4 the same light gives 200 counts and "the same lux value of
   46 lx". Both cannot hold: 200 × 0.2688 is 53.76 lx, so 54 lx is right and the
   46 lx is a slip. The example's own logic — that the lux value is unchanged
   across a gain switch — is correct.

The tables govern in both cases, consistent with §1.

## 9. Threshold monitor

The vendor calls bit 1 an interrupt enable, but the VEML7700 has **no dedicated
interrupt pin**. The observable interface is register `0x06`, polled over I²C.

- bit 15 indicates low-threshold qualification;
- bit 14 indicates high-threshold qualification;
- persistence requires 1, 2, 4, or 8 consecutive qualifying measurements;
- threshold count meaning depends on gain and integration time;
- wall-clock qualification depends on power-saving cadence;
- monitor operation requires the sensor active.

Therefore one `ThresholdMonitorConfig` owns measurement configuration,
thresholds, persistence, power-saving configuration, and active power state.
Programming is disable-first and enable-last.

The official sources do not provide a reliable flag-clearing contract. The v0.1
API exposes observed status only and does not promise read-to-clear,
write-to-clear, or latched GPIO behavior.

- [x] Status register `0x06`: bit 15 `int_th_low`, bit 14 `int_th_high`, both
      read-only, bits 13:0 reserved (Table 7). The driver masks `0x3FFF` and
      rejects any reserved bit set, which this table makes source-backed rather
      than defensive.
- [x] **The qualification condition is vendor-stated, in necessary form.**
      Table 1 establishes `ALS_PERS` and its four values as a *persistence
      protect number*, which is what the driver encodes. Application note 84323
      Rev. 06-Mar-2025, printed page 16, section `INTERRUPT HANDLING`, states
      the condition under which a flag is set:

      > Only when the programmed threshold is exceeded and a programmed number
      > of measurements (`ALS_PERS`) stay above / below this threshold will the
      > corresponding interrupt bit (`ALS_IF_L` or `ALS_IF_H`) be set.

      So `ALS_PERS` counts **measurements that stay above or below the
      threshold**, and *stay* is a continuity condition. That is the word
      *consecutive* in this row's heading, and it is now source-backed rather
      than assumed.

      Note the logical form. "Only when X will Y be set" makes X **necessary**
      for Y; it does not state that X is sufficient. What the passage licenses
      directly is the negative — the flag is not set before the count is
      reached. See #76.

      The datasheet does not carry this. Its Command Code #6 describes a flag
      set by "data crossing" a threshold window, with no persistence condition,
      and Table 7 adds nothing. The rule is application-note-only, which is why
      a search confined to the register definitions missed it for several
      revisions. See #73 and D-032.
- [ ] Whether the stated condition is **sufficient**, and what a non-qualifying
      measurement does to a partial run.

      **Located negative (D-032).** Read: datasheet 84286 Rev. 1.8 — Table 1's
      `ALS_PERS` row, Table 7 *Interrupt Status #6*, and the Command Code #6
      section; application note 84323 Rev. 06-Mar-2025 — `INTERRUPT HANDLING`
      in full and its COMMAND REGISTER FORMAT blocks for `00` and `06`. Those
      sections give the necessary condition quoted above and nothing else about
      qualification: no statement that meeting it always sets the flag, and no
      statement of whether a measurement that fails to qualify resets the count,
      holds it, or decrements it. Absence outside those sections is not claimed.

      This row is **not** an Assumption in the D-029 sense. A qualification rule
      is functional behavior a datasheet can state in prose, unlike an
      oscillator tolerance, so further reading could still close it — and this
      row is the standing proof of that, since further reading closed half of
      what used to be one row.

      Under D-030 it resolves without one. The driver acts **defensively**: it
      encodes the persistence protect number and promises nothing about when the
      flag asserts, which costs it nothing because no driver logic reads the
      count — `Persistence::count()` is an accessor, not an input to any
      computation. The model **declares undefined** rather than assuming,
      because nothing in the model requires a qualification rule either.

      **That allocation is unchanged by the row above, and the reason is worth
      stating.** Predicting when a flag asserts needs the condition to be
      sufficient and needs a reset rule for partial runs. Neither is stated, and
      either gap alone is enough to stop a sound oracle: a model that counts to
      `ALS_PERS` has to decide what a non-qualifying refresh does, and the
      sources do not say. So what changed is the row's premise, not its outcome
      — this is no longer "no rule is stated", it is "the counting condition is
      stated and the rest is not".

      The model previously implemented consecutive counting with reset on any
      non-qualifying refresh, and that was the finding here: driver-model
      conformance appeared to establish persistence semantics when it could not.
      The driver only programs the field, so the trace at persistence 4 confirmed
      a register write while reading like a behavioral result.

      Resolved in #56. `update_threshold_status` qualifies at protect number one
      and reports `Unsupported::UndefinedQualificationRule` above it, so the
      model no longer answers a question the sources never settled. Programming
      any protect number still works, because Table 1 is verified — only
      qualification is withdrawn. That behavior is unchanged by #73; only the
      account of why it is correct has moved.

      Third-party libraries describe this register in terms of an INT pin that
      latches and clears on read. This part has neither — §9 records that the
      interrupt pin is explicitly unavailable, and Table 7 states no clearing
      behavior. Library documentation for family parts is not evidence here.
- [x] The part has no dedicated interrupt pin. The source says so in as many
      words — *Interrupt pin not available for VEML7700* — immediately above the
      register format table, which is why this contract treats `0x06` as a
      polled status word and the API owns no GPIO.
- [x] The sources state no flag-clearing behavior. **The absence is the
      finding.**

      **Located negative (D-032).** Read: datasheet 84286 Rev. 1.8 — Table 7
      *Interrupt Status #6*, the Command Code #6 section, and the `ALS_INT_EN`
      row of Table 1; application note 84323 Rev. 06-Mar-2025 — its
      `INTERRUPT HANDLING` section and its COMMAND REGISTER FORMAT block for
      `06`. Table 7 defines the two flag bits, marks them read-only, and
      reserves bits 13:0. Command Code #6 says each bit "defines interrupt flag
      while trigger occurred due to data crossing" its threshold window. The app
      note's `INTERRUPT HANDLING` section states the condition under which a bit
      is **set** and says nothing about when or how one is cleared. Nothing in
      those sections describes read-to-clear, write-to-clear, or any other
      deassertion.

      Those are the places such a rule would be stated — a register's own
      definition and the section devoted to using the feature. The v0.1 API
      therefore promises observation only, which is D-010.

## 10. Identity and support claim

At fixed 7-bit address `0x10`, the ID register is expected to transfer bytes
`0x81, 0xC4`, decoded as word `0xC481`. `probe()` distinguishes:

- address NACK: not present;
- other bus error: preserved concrete error;
- readable wrong ID: wrong device;
- exact ID: compatible with this driver contract.

This is compatibility evidence, not package-orientation, lot, authenticity, or
calibration proof.

- [x] The ID register transfers bytes `0x81, 0xC4`, decoding to `0xC481`, at the
      fixed address option (Table 8). The low byte is the fixed device ID
      `0x81`; the high byte is an **address-option code**, `0xC4` for slave
      address `0x20` and `0xD4` for `0x90`. `0x20` is the 8-bit write form of the
      7-bit `0x10` this driver fixes, so `0xC481` is the word for the supported
      option — and `0xD481` is a real VEML7700 at an address this driver does not
      support, which is why identity is a compatibility claim rather than a
      presence claim.

      Byte order follows §3: low first, so `0x81` then `0xC4` on the wire.

## 11. Explicit non-claims

The driver does not claim:

- calibrated lux at the system aperture;
- cover-glass/window compensation;
- source-spectrum or cosine-response correction;
- automatic range selection;
- empirical high-lux correction;
- simultaneous ALS/white conversion or atomic paired register read;
- a physical interrupt output;
- safety or metrology certification.
