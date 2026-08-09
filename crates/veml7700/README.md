# ph-veml7700-als

Async, allocation-free VEML7700 driver scaffold.

```rust,no_run
use ph_veml7700_als::{MeasurementConfig, Veml7700};
use embedded_hal_async::{delay::DelayNs, i2c::I2c};

async fn sample<I2C: I2c, D: DelayNs>(
    i2c: I2C,
    delay: &mut D,
) -> Result<(), ph_veml7700_als::MeasureOnceError<I2C::Error>> {
    let mut sensor = Veml7700::new(i2c);
    let reading = sensor.measure_once(delay, MeasurementConfig::safe_bright_start()).await?;
    let _micro_lux = reading.nominal_illuminance.as_micro_lux();
    Ok(())
}
```

Nominal lux conversion does not compensate for a cover window, fixture geometry,
source spectrum, part tolerance, or the vendor's application-dependent high-lux
correction. See the repository contracts before publication.
