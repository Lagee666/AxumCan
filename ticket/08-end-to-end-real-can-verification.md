# 08: End-to-end real-CAN verification

**What to build:** The project provides a repeatable verification path proving that one explicit CAN Model produces the expected bytes in Print mode and the same bytes through Linux SocketCAN when an interface is available.

**Blocked by:** 04: Linux SocketCAN transport mode; 07: Startup validation and sender lifecycle hardening.

**Status:** ready-for-agent

- [x] A minimal explicit CAN Model defines one Channel, one Message, and representative Signals.
- [x] Print mode shows the expected Channel, CAN ID, identifier type, and eight-byte payload.
- [x] SocketCAN mode sends the same identifier and payload to the Channel-named interface.
- [x] The verification path does not require physical CAN hardware.
- [x] The verification path clearly reports when the required Linux interface is unavailable.
- [x] The documented or automated check confirms periodic transmission and runtime Signal Value updates.
