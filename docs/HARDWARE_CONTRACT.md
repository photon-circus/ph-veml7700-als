# VEML7700 hardware contract

Binding interpreted device facts for `ph-veml7700-als`. A checked row means the
owner has verified the recorded official Vishay source. Unchecked rows remain
provisional and must not be promoted to physical-support claims.

## 1. Source baseline

| Source | Revision | Status |
| --- | --- | --- |
| VEML7700 datasheet, document 84286 | Rev. 1.8, 28-Nov-2024 | [ ] owner verified |
| Designing the VEML7700 Into an Application, document 84323 | 06-Mar-2025 | [ ] owner verified |

The datasheet and application note occasionally use inconsistent prose such as
“six registers” while their command table includes `0x00` through `0x07` and a
power-saving register at `0x03`. The explicit command/register tables and ID
section govern this driver; the discrepancy remains documented rather than
silently normalized.

## 2. Electrical and bus boundary

- [ ] VDD operating range is 2.5 V to 3.6 V.
- [ ] I²C bus high-level supply may be 1.7 V to 3.6 V.
- [ ] Standard and fast mode are supported from 10 kHz through 400 kHz.
- [ ] The fixed 7-bit address is `0x10` (`0x20` write / `0x21` read in 8-bit form).
- [ ] Pull-ups are external; the vendor suggests values above 1 kΩ, commonly
      2.2 kΩ to 4.7 kΩ.
- [ ] The driver never owns sensor power, board pull-ups, cover-window geometry,
      or an external optical source.

## 3. Word transfer order

Every command register is 16 bits. The I²C protocol transfers **data byte low,
then data byte high** for both writes and reads.

Consequences:

- `u16::from_le_bytes([low, high])` is mandatory on reads;
- `value.to_le_bytes()` is mandatory on writes;
- a big-endian register helper is a review-blocking defect;
- strict transaction tests inspect exact byte order.

## 4. Register map

| Pointer | Semantic name | Access | Driver treatment | Reset value |
| --- | --- | --- | --- | --- |
| `0x00` | configuration | R/W | strict typed codec | `0x0001` |
| `0x01` | high threshold | R/W | raw ALS counts | not declared by sources |
| `0x02` | low threshold | R/W | raw ALS counts | not declared by sources |
| `0x03` | power saving | R/W | strict typed codec | `0x0000` |
| `0x04` | ALS output | R | snapshot/fresh result | not declared by sources |
| `0x05` | white output | R | raw spectral companion channel | not declared by sources |
| `0x06` | threshold status | R | bit 15 low, bit 14 high | not declared by sources |
| `0x07` | ID | R | expected word `0xC481` at the fixed address option | source-declared identity `0xC481` |

No public raw-register accessor exists in v0.1.

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

## 7. Wake-up, integration, and freshness

- after changing shutdown bit from 1 to 0, wait at least 2.5 ms before the first
  measurement;
- integration time has an assumed ±30 % tolerance;
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

The core uses exact integer micro-lux-per-count values. It does not apply the
vendor's empirical high-illuminance polynomial because its applicability depends
on optical window, source spectrum, geometry, and application validation.

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

## 10. Identity and support claim

At fixed 7-bit address `0x10`, the ID register is expected to transfer bytes
`0x81, 0xC4`, decoded as word `0xC481`. `probe()` distinguishes:

- address NACK: not present;
- other bus error: preserved concrete error;
- readable wrong ID: wrong device;
- exact ID: compatible with this driver contract.

This is compatibility evidence, not package-orientation, lot, authenticity, or
calibration proof.

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
