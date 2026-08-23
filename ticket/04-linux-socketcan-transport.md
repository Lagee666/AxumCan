# 04: Linux SocketCAN transport mode

**What to build:** The simulator can send the same encoded classic CAN Frames that Print mode displays to Linux SocketCAN interfaces named by their CAN Channels.

**Blocked by:** 01: Explicit CAN Model with Print-mode transmission; 02: Complete classic CAN Signal encoding.

**Status:** ready-for-agent

- [ ] SocketCAN mode is selected during Rust application initialization.
- [ ] Each configured CAN Channel is opened as the corresponding Linux SocketCAN interface.
- [ ] The SocketCAN transport receives the project-owned encoded CAN Frame without re-encoding Signal data.
- [ ] Standard and extended identifiers are delivered correctly.
- [ ] Initialization fails if any required interface cannot be opened.
- [ ] SocketCAN mode never silently falls back to Print mode.
- [ ] With an available `vcan` interface, an end-to-end test or documented verification observes the expected CAN ID and bytes.
