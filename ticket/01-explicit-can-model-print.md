# 01: Explicit CAN Model with Print-mode transmission

**What to build:** A startup-loaded canonical CAN Model can describe one explicit classic CAN Message and periodically print its encoded CAN Frame.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] A public canonical model represents Channels, Messages, Signals, CAN IDs, Cycle Times, and initial Signal Values.
- [ ] An asynchronous model source loads the model at startup.
- [ ] The existing shorthand JSON can be translated into the canonical model through a compatibility source.
- [ ] One unsigned little-endian Signal can be encoded into a validated classic CAN Frame.
- [ ] Print mode displays the Channel, CAN ID, extended-ID state, and frame bytes.
- [ ] The Message transmits periodically using its configured Cycle Time.
- [ ] A fake source and print transport provide an end-to-end test of startup, encoding, and periodic output.
