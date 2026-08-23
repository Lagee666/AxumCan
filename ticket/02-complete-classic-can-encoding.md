# 02: Complete classic CAN Signal encoding

**What to build:** Signals can use real CAN layout and physical-value rules while invalid models fail before transmission.

**Blocked by:** 01: Explicit CAN Model with Print-mode transmission.

**Status:** ready-for-agent

- [ ] Signals support start bit, bit length, little-endian and big-endian byte order, signedness, factor, offset, limits, and initial physical values.
- [ ] Physical values are converted to raw values using factor and offset.
- [ ] Standard and extended CAN IDs are validated correctly.
- [ ] Classic CAN payloads are limited to eight bytes.
- [ ] Overlapping, out-of-bounds, and otherwise invalid Signal layouts are rejected during startup.
- [ ] Invalid factors, limits, Cycle Times, and Signal values are rejected before sender tasks start.
- [ ] Golden tests cover multi-byte, byte-crossing, signed, scaled, and big-endian Signals.
