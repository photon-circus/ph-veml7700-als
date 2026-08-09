# Managed VEML7700 HIL runner

Classic ESP32 firmware managed by `ph-hil`. It is excluded from the driver
workspace and must exercise the public `ph-veml7700-als` API through a board/BSP
adapter.

The scaffold leaves HAL construction, pin mapping, switchable VDD, optical-source
control, reference-instrument synchronization, and dutlink-v1 transport
unfinished. Keep the dispatcher capability inventory exactly equal to
`capabilities.txt`.

Required safe boot state:

- sensor VDD off;
- SDA/SCL and source controls released;
- optical source disabled;
- captures inactive;
- dutlink-v1 banner emitted only after safing completes.

There is no interrupt GPIO for this device.
