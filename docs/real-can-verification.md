# Real CAN verification

AxumCan can run without CAN support by selecting `TransportMode::Print`. Print mode emits the Channel, CAN ID, identifier type, and encoded bytes that SocketCAN would transmit.

To verify Linux SocketCAN delivery without physical hardware, create a virtual CAN interface:

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set up vcan0
```

Configure a CAN Model whose Channel is `vcan0`, select `TransportMode::SocketCan` during Rust initialization, and start the simulator. Observe the frames with:

```bash
candump vcan0
```

The CAN ID and data bytes reported by `candump` must match Print mode for the same model and Signal Values. If the interface cannot be opened, initialization must fail rather than silently switching to Print mode.

Change a Signal Value through the dashboard while `candump` is running. The next periodic frame should contain the updated encoded value, demonstrating latest-state propagation from the Dashboard through the CAN Simulator to the CAN Channel.

Clean up the virtual interface when finished:

```bash
sudo ip link delete vcan0
```
