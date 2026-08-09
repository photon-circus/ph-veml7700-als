# Crate architecture

The normative architecture is maintained in `../../docs/ARCHITECTURE.md`.

The publishable crate contains one direct `Veml7700<I2C>` facade, private
register definitions, pure configuration/power/scaling/threshold codecs,
contextual errors, strict host test transports, and an autonomous behavioral
model. Module paths are not public API; users import from the crate root.
