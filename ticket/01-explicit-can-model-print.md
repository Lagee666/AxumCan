# 01: Explicit CAN Model with Print-mode transmission

**What to build:** A startup-loaded canonical CAN Model can describe one explicit classic CAN Message and periodically print its encoded CAN Frame.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [x] A public canonical model represents Channels, Messages, Signals, CAN IDs, Cycle Times, and initial Signal Values.
- [x] An asynchronous model source loads the model at startup.
- [x] The existing shorthand JSON can be translated into the canonical model through a compatibility source.
- [x] One unsigned little-endian Signal can be encoded into a validated classic CAN Frame.
- [x] Print mode displays the Channel, CAN ID, extended-ID state, and frame bytes.
- [x] The Message transmits periodically using its configured Cycle Time.
- [x] A fake source and transport provide an end-to-end test of startup, encoding, and periodic output.
