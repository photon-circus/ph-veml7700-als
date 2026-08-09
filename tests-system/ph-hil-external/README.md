# External ph-hil integration boundary

These tests run with a separately installed `ph-hil` and validate the schema-1
build hook, plan, mock bench, transcript, retained artifacts, and assessment
policy. They must not add `ph-hil` as a dependency of the publishable driver
crate.
