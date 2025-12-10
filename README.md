# Rodecaster Pro II Control Bridge

This project enables programmatic control of the **Rodecaster Pro II** by intercepting and injecting commands into its internal TCP communication channel. It allows you to control fader mutes, input sources, and virtual levels from the command line or scripts, while ensuring the device's screen remains perfectly synchronized.

## Features

*   **Fader Control**: Mute/Unmute any virtual fader.
*   **Source Routing**: Change input sources (e.g., assign "Soundpad" to Fader 1) on the fly.
*   **Virtual Levels**: Adjust volume for virtual faders.
*   **UI Synchronization**: Uses a "Loopback Injection" technique to ensure the Rodecaster's touchscreen updates instantly to reflect your changes.

## Components

The project consists of two main Rust binaries rooted in `tcp-bridge/`:

1.  **`tcp-bridge`**: A TCP Proxy that sits between the UI App and the Audio Engine.
    *   Intercepts traffic on `127.0.0.1:2345`.
    *   Injects commands via a local control socket.
2.  **`bridge-ctl`**: A CLI tool to send commands to the bridge.

## Getting Started

### Prerequisites

*   **Rust**: Stable toolchain.
*   **Cross**: For cross-compiling to `aarch64-unknown-linux-musl` (`cargo install cross`).
*   **SSH Access**: You must have root SSH access to your Rodecaster.

### Deployment

Use the provided PowerShell script to build and deploy everything to the device:

```powershell
.\deploy.ps1
```

This will:
1.  Cross-compile `tcp-bridge` and `bridge-ctl`.
2.  SCP the binaries and scripts (`run-proxy.sh`) to `/tmp/` on the device.

### Running

1.  SSH into your Rodecaster.
2.  Run the proxy startup script:
    ```bash
    /tmp/run-proxy.sh
    ```
    *This sets up the network alias, configures iptables redirection, and starts the bridge.*

### Usage

Open a second SSH terminal (or use `socat` from scripts) to control the device using `bridge-ctl`.

**Examples:**

```bash
# Mute Physical Fader 1 (Index 0)
/tmp/bridge-ctl mute 0 1

# Unmute Virtual Fader 1 (Index 6)
/tmp/bridge-ctl mute 6 0

# Set Physical Fader 1 Source to Soundpad (ID 11)
/tmp/bridge-ctl source 0 11

# Set Virtual Fader 2 Level to 75/127
/tmp/bridge-ctl level 7 75
```

## Documentation

For a deep dive into the reverse-engineered protocol, packet structures, and mapping tables, see [PROTOCOL_AND_BRIDGE.md](PROTOCOL_AND_BRIDGE.md).
