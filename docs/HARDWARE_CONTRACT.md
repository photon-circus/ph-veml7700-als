# VEML7700 evidence registry

> **Authority: shared evidence registry.** Each `S-nn` permanently identifies
> one scoped proposition and the evidence that addresses it. The identifier is
> the proposition's only prose authority outside this file.

This registry is descriptive, never prescriptive. It records no driver policy,
model policy, sufficiency judgment, or shared inference. The driver and model
cite the same referent and derive their consequences independently. Source
coordinates and physical artifacts are recorded here once, not copied beside
those reactions.

## Registry semantics

- An identifier is never reused or redefined. If a proposition changes or
  splits, its old row remains resolvable and each new proposition receives a
  new ID.
- A **documentary proposition** records what a pinned source states, omits, or
  contradicts. A **device proposition** records behavior attributed to scoped
  silicon. Hardware cannot refute what a document says; a silicon discrepancy
  receives its own linked device proposition.
- Evidence is labelled **positive** when it supports a proposition and
  **negative** when it refutes a device proposition or supports a located
  documentary omission. **Undefined** is a device-proposition knowledge state,
  not evidence polarity and not an opposite behavior.
- A documentary omission cannot stand in for silicon behavior. Behavior that
  evidence does not determine has its own truth-apt device proposition, marked
  undefined, so later physical evidence can support or refute that exact
  referent.
- Evidence is appended as `supports`, `refutes`, or `does not resolve` and never
  overwrites contrary history. Physical evidence names its observed population,
  conditions, procedure, and durable artifact.
- **Relevance** is mutable legacy metadata, not part of a proposition or its
  evidence. `Not currently relevant` preserves a pre-existing ID that has no
  current driver, model, conformance, scoped-hardware, or reported-bug
  consumer; it creates no work or coverage obligation.
- A **registry tombstone** is the row an identifier leaves behind when it stops
  naming a live proposition. Because an ID is never reused, the tombstone is
  what keeps an existing citation resolvable. Its state is `superseded` when one
  or more named `S-nn` now carry the referent, and `retired` when none do.
  Evidence is always `registry history only`: a tombstone asserts nothing about
  documents or silicon, so it can be neither supported nor undefined, and
  `Relevance` does not apply to it.

Every row names its proposition kind, current knowledge state, and evidence
polarity. `Supported` means the cited evidence supports that exact proposition;
`undefined` means the cited evidence does not determine it. States report
evidence, not approval or future work.

A tombstone additionally names the **former proposition** it used to carry, so a
reader arriving from an old citation learns what the identifier once meant
without recovering the file's history.

Rows contain propositions, evidence, and state only. Component consequences and
work-item policy belong elsewhere. Cite an `S-nn`, never a movable section
number, and do not reproduce its proposition beside the citation.

## 1. Source baseline

| Source | Revision |
| --- | --- |
| VEML7700 datasheet, document 84286 | Rev. 1.8, 28-Nov-2024 |
| Designing the VEML7700 Into an Application, document 84323 | 06-Mar-2025 |

[`docs/vendor/README.md`](vendor/README.md) records the source digests. This
binds the registry to retrieved bytes; it does not establish silicon behavior.

## 2. Electrical and bus boundary



### S-01

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Sensor supply `V_DD` operating range is 2.5 V to 3.6 V. The
Product Summary names this `OPERATING VOLTAGE RANGE`; Basic
Characteristics gives `V_DD` as MIN 2.5 V, TYP 3.3 V, MAX 3.6 V.
**Relevance: not currently relevant.**

### S-02

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The I²C pull-up supply range is 1.7 V to 3.6 V. Datasheet
84286 Rev. 1.8,
page 1, `PRODUCT SUMMARY`, and application note 84323 (06-Mar-2025),
page 2, *Application Circuitry for the VEML7700*.

**Scope:** the pull-up supply rail only.
**Relevance: not currently relevant.**

### S-03

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** I²C bus **input** H-level range `V_ih` is 1.3 V to 3.6 V, and
input L-level range `V_il` is −0.3 V to 0.4 V, both specified at `V_DD` =
3.3 V (Basic Characteristics). These are signal thresholds on
`SCL`/`SDA`, **not** a supply. The datasheet's I²C Interface section
restates the first as "I²C H-level range = 1.3 V to 3.6 V".

**Relevance: not currently relevant.**

### S-04

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Clock frequency `f(SMBCLK)` is 10 kHz to 100 kHz in standard mode
and 10 kHz to 400 kHz in fast mode (*I²C Timing Characteristics*). The
source marks these values as based on the standard I²C protocol
requirement and **not tested in production**.

**Relevance: not currently relevant.**

### S-45

**Kind:** documentary. **State:** supported. **Evidence:** positive documentary conflict.

**Proposition:** The sources give `f(SCL)` two different ways.
*Basic Characteristics* states a flat 10 kHz to 400 kHz with no mode
split, while *I²C Timing Characteristics* splits standard mode at 100 kHz
from fast mode at 400 kHz (`S-04`). The registry records the conflict without
resolving it.

**Relevance: not currently relevant.**

### S-05

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The fixed 7-bit address is `0x10` (`0x20` write / `0x21` read in
8-bit form).

### S-06

**Kind:** device. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** Pull-ups are external; the vendor suggests values above 1 kΩ,
commonly 2.2 kΩ to 4.7 kΩ.

**Relevance: not currently relevant.**

### S-07

**Kind:** registry tombstone. **State:** retired. **Evidence:** registry history only.

**Former proposition:** a component policy rule, not an evidence proposition.

No `S-nn` carries the referent, because policy never belonged in this registry.
The identifier remains resolvable.

## 3. Word transfer order

### S-08

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Low byte then high byte for every 16-bit register, on reads and
writes. Fig. 9 shows both frames explicitly: a write sends command code,
then `Data byte (LSB)`, then `Data byte (MSB)`; a read returns them in
the same order after the repeated start.

## 4. Register map

### S-09

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Pointer values and access direction for all eight registers, from
the COMMAND REGISTER FORMAT table: `00` ALS_CONF_0 R/W, `01` ALS_WH R/W,
`02` ALS_WL R/W, `03` Power saving R/W, `04` ALS R, `05` WHITE R, `06`
ALS_INT R, `07` ID R. Separate prose calls `03h` "not defined" while this
table defines it as power saving.

### S-10

**Kind:** documentary. **State:** supported. **Evidence:** located negative.

**Proposition:** The pinned sources declare no full
power-on word for registers `0x01` through `0x06`. Register `0x03` is
partial: `S-48` covers its `PSM` field while `S-11` records the undefined
remainder.

**Located negative:** read datasheet 84286 Rev. 1.8 — the COMMAND REGISTER FORMAT
overview, the per-register Command Code #0 through #7 sections, and
Tables 1, 4, 7 and 8; application note 84323 Rev. 06-Mar-2025 — every
COMMAND REGISTER FORMAT block it repeats. Those scopes declare no full
power-on word for `0x01` through `0x06`.



### S-48

**Kind:** device. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** The `PSM` field comes up as mode 1 (`00`) before it is written.
Application note 84323 Rev. 06-Mar-2025, in the power-saving discussion
preceding its *Command Code PSM / PSM_EN* register-format table: describing
how to enable the feature, it states that the default it comes up with is
mode 1 = `00` for bits 2 and 1.


### S-11

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** Reserved bits 15:3 and `PSM_EN`
all read `0` before `0x03` is written, and with them the full word reads
`0x0000`.

**Documentary evidence: does not resolve (located negative).** Read:
datasheet 84286 Rev. 1.8, the
command-register overview table, Table 4 *Power Saving Modes*, and the
register-format note that declares a power-on default for command code 0;
application note 84323 Rev. 06-Mar-2025, the *Command Code PSM / PSM_EN*
register-format table and the power-saving discussion preceding it. Those
scopes state no power-on value for `PSM_EN` or the reserved bits.

**Physical evidence: none.**

## 5. Configuration register `0x00`

### S-12

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Reset/default word `0x0001`. The COMMAND REGISTER FORMAT note
states it directly: *command code 0 default value is 01 = devices is shut
down*. **Table 1 does not establish this;** it gives bit meanings and no
power-on value.

**Located negative:** datasheet 84286 Rev. 1.8, Table 1 *ALS_CONF_0 #0*
and its surrounding Command Code #0 section contain no reset column; the
default occurs only in the adjacent register-format note.

### S-13

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Reserved bits 15:13 are `000b`.

### S-14

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Gain encodings: `00` ×1, `01` ×2, `10` ×1/8, `11` ×1/4. Note that
the encoding order is not the magnitude order — `10` is ×1/8 and `11` is
×1/4, so a table sorted by gain does not match a table sorted by bit
pattern.


### S-15

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Integration-time encodings, bits 9:6: `1100` 25 ms, `1000` 50 ms,
`0000` 100 ms, `0001` 200 ms, `0010` 400 ms, `0011` 800 ms. Like gain,
the encoding order is not the magnitude order.

### S-16

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Persistence encodings, bits 5:4: `00` 1, `01` 2, `10` 4, `11` 8
(Table 1, `ALS_PERS`).

### S-17

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Monitor-enable, bit 1 (`ALS_INT_EN`: 0 disable, 1 enable), and
shutdown, bit 0 (`ALS_SD`: 0 power on, 1 shut down), from Table 1.

### S-18

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Reserved bits: 15:13 `000b`, bit 10 `0b`, and 3:2 `00b` (Table 1).

### S-19

**Kind:** registry tombstone. **State:** retired. **Evidence:** registry history only.

**Former proposition:** Reconfiguration requires shutdown first, as a device
proposition.

No `S-nn` carries that referent. The cited evidence establishes only the
distinct documentary proposition now identified by `S-56` — what the vendor's
example flow does, not what the silicon requires — so `S-56` is not a
replacement, and neither row is evidence for a component consequence.

### S-56

**Kind:** documentary. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** The vendor's example software flow sets
`ALS_SD = 1` before changing gain or integration time, then clears
`ALS_SD` afterward.

**Provenance:** Application note 84323, revision 06-Mar-2025, printed page 21,
Fig. 23, *Flow Chart with Correction Formula from at least 100 lx*.

## 6. Power-saving register `0x03`

### S-20

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Register `0x03` field layout (Table 4): bits 15:3 reserved, bits
2:1 `PSM` selecting `00` mode 1 through `11` mode 4, bit 0 `PSM_EN` (0
disable, 1 enable).

### S-21

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** At gain ×2, the vendor's refresh-time / I_DD / resolution relation
gives these sixteen refresh times:

| Mode | 100 ms | 200 ms | 400 ms | 800 ms |
| --- | ---: | ---: | ---: | ---: |
| 1 | 600 ms | 700 ms | 900 ms | 1300 ms |
| 2 | 1100 ms | 1200 ms | 1400 ms | 1800 ms |
| 3 | 2100 ms | 2200 ms | 2400 ms | 2800 ms |
| 4 | 4100 ms | 4200 ms | 4400 ms | 4800 ms |

### S-44

**Kind:** documentary. **State:** supported. **Evidence:** located negative.

**Proposition:** No refresh time is documented for 25 ms or 50 ms integration.
The absence is the finding.

**Located negative.** Read: the *Refresh Time, I_DD, and Resolution
Relation* table in datasheet 84286 Rev. 1.8 and its identical copy in
application note 84323 Rev. 06-Mar-2025, plus the app note's separate
`PSM` / `ALS_IT` → refresh-time table. All three are indexed by `ALS_IT` ∈
{100, 200, 400, 800} ms; neither 25 ms nor 50 ms appears as a row in any of
them, in either document. Absence outside those tables is not claimed.


### S-22

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** Refresh time is independent of ALS gain.

**Documentary evidence: does not resolve (located negative).** Read:
datasheet 84286 Rev. 1.8, section
*Refresh Time Determination of PSM* and the *Refresh Time, I_DD, and
Resolution Relation* table; application note 84323 Rev. 06-Mar-2025, the
power-saving discussion at *Command Code PSM / PSM_EN* and its own copy of
the *Refresh Time, I_DD, and Resolution Relation* table. Neither document
states gain dependence or gain independence.

**Physical evidence: none.**

## 7. Wake-up and conversion timing

### S-23

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The 2.5 ms minimum wake-up delay after clearing the shutdown bit.
The source's flow chart states `ALS_SD = 0`, then wait ≥ 2.5 ms.

### S-24

**Kind:** documentary. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** The vendor states that a ±30 % integration-time tolerance can be
assumed, and that it should be considered when reading measurement
results. Application note 84323, Revision 06-Mar-2025, page 4, section
*Command Code ALS_IT*, `Remark`:

> For the integration time a tolerance of ± 30 % can be assumed. This
> tolerance should also be considered during the read out of the
> measurement results.

**Evidence classification:** positive vendor guidance. A located review of
the datasheet's *Absolute Maximum Ratings* and *Basic Characteristics*
found no characterized integration-time or oscillator-accuracy limit.

### S-55

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** With power saving disabled, a completed conversion is available
no later than the `S-23` wake interval plus 130 % of the selected nominal
integration time.

**Documentary evidence: does not resolve.** `S-24` states the ±30 % figure as an
allowance that *can be assumed*, not as a min/max characterized across process,
voltage, and temperature, so it does not determine this bound. `S-23` supplies
the wake interval only. Neither proposition is evidence that the composed bound
holds. **Physical evidence: none.**



### S-25

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Data registers retain the last result while shut down. The source
calls this *Auto-Memorization*: the part memorizes the last ambient data
before shutdown, the host may read it directly while shut down, and on
wake the data is refreshed by a new detection.

## 8. ALS and white channels

### S-26

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The complete twenty-four-entry resolution table, every gain and
every integration time:

| IT | gain ×2 | gain ×1 | gain ×1/4 | gain ×1/8 | unit |
| ---: | ---: | ---: | ---: | ---: | --- |
| 800 ms | 0.0042 | 0.0084 | 0.0336 | 0.0672 | lx/count |
| 400 ms | 0.0084 | 0.0168 | 0.0672 | 0.1344 | lx/count |
| 200 ms | 0.0168 | 0.0336 | 0.1344 | 0.2688 | lx/count |
| 100 ms | 0.0336 | 0.0672 | 0.2688 | 0.5376 | lx/count |
| 50 ms | 0.0672 | 0.1344 | 0.5376 | 1.0752 | lx/count |
| 25 ms | 0.1344 | 0.2688 | 1.0752 | 2.1504 | lx/count |

### S-27

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The twenty-four-entry maximum-detection-range table:

| IT | gain ×2 | gain ×1 | gain ×1/4 | gain ×1/8 | unit |
| ---: | ---: | ---: | ---: | ---: | --- |
| 800 ms | 275 | 550 | 2 202 | 4 404 | lx |
| 400 ms | 550 | 1 101 | 4 404 | 8 808 | lx |
| 200 ms | 1 101 | 2 202 | 8 808 | 17 616 | lx |
| 100 ms | 2 202 | 4 404 | 17 616 | 35 232 | lx |
| 50 ms | 4 404 | 8 808 | 35 232 | 70 463 | lx |
| 25 ms | 8 808 | 17 616 | 70 463 | 140 926 | lx |

### S-28

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Gain ×1/8 at 25 ms is 2.1504 lx/count, reaching 140 926 lx — the
widest range in `S-26` and `S-27`. Basic Characteristics names the same
pair as the *detectable maximum illuminance* condition, `E_V max` =
140 000 lx.

### S-29

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Above 100 lx, gain ×1 and ×2 are outside the linear region.

### S-30

**Kind:** documentary. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** Correction is called for with gain ×1/4 and ×1/8, and above 1 000
lx.

### S-31

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The correction polynomial coefficients, checked against the
source's own worked example: 5581 counts at ×1/4 and 100 ms give 1500 lx
uncorrected and 1658 lx corrected:

```text
corrected = a·x⁴ + b·x³ + c·x² + d·x
a = 6.0135e-13   b = -9.3924e-9   c = 8.1488e-5   d = 1.0023
```

### S-32

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** ALS output is a 16-bit word.

### S-51

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** When the internal ALS conversion exceeds the 16-bit code range,
the output clips at `0xFFFF` rather than wrapping or using another
indication.

**Documentary evidence: does not resolve (located negative).** Read:
datasheet 84286 Rev. 1.8 — *Product
Summary*, *Basic Characteristics*, the register tables and Command Code
#4; application note 84323 Rev. 06-Mar-2025 — *Resolution and Maximum
Detection Range*, Fig. 23's correction flow, and its `04` register block.
Those passages establish the 16-bit output and nominal maximum-detection
ranges but do not define an overflow flag or assign clipping semantics to
exactly 65 535. Absence outside those sections is not claimed.
**Physical evidence: none.**

### S-52

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** An ALS output of `0xFFFF` establishes that the incident scene
reached or exceeded the selected nominal full-scale range during that
conversion.

**Documentary evidence: does not resolve.** The located review and scope
recorded under `S-51` also found no lower-bound meaning assigned to the
maximum code. **Physical evidence: none.**

### S-33

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The white channel is a 16-bit count: command code `05` is defined
as *MSB 8 bits data of whole WHITE 16 bits* and *LSB 8 bits*, read-only,
with all sixteen bits carrying data. No sign bit is defined for it or for
ALS, and no passage describes either as signed.

**Located negative** for that last sentence. Read: datasheet 84286
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

### S-34

**Kind:** documentary. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** Start at the lowest gain for unknown brightness, and reduce
integration time below 100 ms to cover the brightest conditions.

### S-35

**Kind:** registry tombstone. **State:** superseded. **Evidence:** registry history only.

**Former proposition:** a repetition of the proposition now owned by `S-29`.

### S-36

**Kind:** registry tombstone. **State:** superseded. **Evidence:** registry history only.

**Former proposition:** a summary spanning parts of `S-26`, `S-29`, and `S-30`,
which now carry the referent separately.

### S-37

**Kind:** documentary. **State:** supported. **Evidence:** positive vendor guidance.

**Proposition:** Auto-ranging is application-software responsibility in the source.

### S-46

**Kind:** documentary. **State:** supported. **Evidence:** positive documentary conflict.

**Proposition:** The narrative's range for 0.0042 lx/count disagrees with the
table. The prose calls it "approximately 0 lx to 230 lx"; the
maximum-detection table gives 275 lx for that pair (`S-27`).

**Relevance: not currently relevant.**

### S-47

**Kind:** documentary. **State:** supported. **Evidence:** positive documentary conflict.

**Proposition:** The ranging example contradicts its own arithmetic. It states
that 100 counts at ×1/8 is 54 lx, then that after
switching to ×1/4 the same light gives 200 counts and "the same lux value
of 46 lx". The resolution table gives 200 × 0.2688 = 53.76 lx (`S-26`).

**Relevance: not currently relevant.**

## 9. Threshold monitor

### S-38

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** Status register `0x06`: bit 15 `int_th_low`, bit 14 `int_th_high`,
both read-only, bits 13:0 reserved (Table 7).

### S-39

**Kind:** documentary. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The vendor states a qualification condition. Table 1
establishes `ALS_PERS` and its four values as a *persistence protect
number*. Application note 84323 Rev. 06-Mar-2025, printed page 16, section
`INTERRUPT HANDLING`, states:

> Only when the programmed threshold is exceeded and a programmed number
> of measurements (`ALS_PERS`) stay above / below this threshold will the
> corresponding interrupt bit (`ALS_IF_L` or `ALS_IF_H`) be set.

The datasheet does not carry this. Its Command Code #6 describes a flag
set by "data crossing" a threshold window, with no persistence condition,
and Table 7 adds nothing. The condition is application-note-only.

### S-40

**Kind:** registry tombstone. **State:** superseded. **Evidence:** registry history only.

**Former proposition:** persistence sufficiency and partial-run behavior
combined in one row. The two independently correctable device propositions are
now `S-49` and `S-50`.

### S-49

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** Satisfying the qualification condition in `S-39` is sufficient
to set the corresponding threshold flag.

**Documentary evidence: does not resolve (located negative).** Read:
datasheet 84286 Rev. 1.8 — Table 1's
`ALS_PERS` row, Table 7 *Interrupt Status #6*, and the Command Code #6
section; application note 84323 Rev. 06-Mar-2025 — `INTERRUPT HANDLING`
in full and its COMMAND REGISTER FORMAT blocks for `00` and `06`. Those
sections state the necessary condition in `S-39` but do not state that
meeting it always sets the flag. Absence outside those sections is not
claimed. **Physical evidence: none.**

### S-50

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** A non-qualifying measurement resets any partial persistence run
to zero.

**Documentary evidence: does not resolve (located negative).** The
same sections enumerated for `S-49` do not
state whether a non-qualifying measurement resets, holds, or decrements a
partial count. Absence outside those sections is not claimed.
**Physical evidence: none.**

### S-41

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The part has no dedicated interrupt pin. The source says so in as
many words — *Interrupt pin not available for VEML7700* — immediately
above the register format table.

### S-42

**Kind:** documentary. **State:** supported. **Evidence:** located negative.

**Proposition:** The sources state no flag-clearing behavior.

**Located negative.** Read: datasheet 84286 Rev. 1.8 — Table 7
*Interrupt Status #6*, the Command Code #6 section, and the `ALS_INT_EN`
row of Table 1; application note 84323 Rev. 06-Mar-2025 — its
`INTERRUPT HANDLING` section and its COMMAND REGISTER FORMAT block for
`06`. Those scopes state no read-to-clear, write-to-clear, or other
deassertion behavior.

### S-53

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** Reading threshold-status register `0x06` leaves both flag bits
unchanged.

**Documentary evidence: does not resolve.** `S-42` records the applicable
located negative. **Physical evidence: none.**

### S-54

**Kind:** device. **State:** undefined. **Evidence:** does not resolve.

**Proposition:** Disabling or re-enabling threshold monitoring leaves both flag
bits unchanged.

**Documentary evidence: does not resolve.** `S-42` records the applicable
located negative. **Physical evidence: none.**

## 10. Identity



### S-43

**Kind:** device. **State:** supported. **Evidence:** positive vendor evidence.

**Proposition:** The ID register transfers bytes `0x81, 0xC4`, decoding to
`0xC481`, at the fixed address option (Table 8). The low byte is the
fixed device ID `0x81`; the high byte is an **address-option code**,
`0xC4` for slave address `0x20` and `0xD4` for `0x90`. `0x20` is the
8-bit write form of 7-bit address `0x10`; `0x90` is the 8-bit write form
of 7-bit address `0x48`.
