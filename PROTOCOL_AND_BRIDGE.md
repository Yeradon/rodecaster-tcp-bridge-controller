# Rodecaster Pro II: Protocol & Bridge Documentation

## Overview
This project provides a complete solution for programmatic control of the Rodecaster Pro II. It uses a **TCP Proxy** to intercept and inject commands into the internal communication channel between the UI (`rc_gui_juce`) and the Audio Engine (`rc_audio_mixer`), enabling features like Fader Mute, Input Source Selection, and Level Control, with full UI synchronization.

## 1. Architecture: TCP Proxy
The communication occurs over a TCP connection on `127.0.0.1:2345`.
We replace this with a Proxy:
1.  **Intercept**: The Proxy binds to `127.0.0.2:9000` and uses `iptables` (DNAT) to redirect traffic destined for `127.0.0.1:2345` to itself.
2.  **Forward**: It connects to the real Server (`127.0.0.1:2345`) and bridges the traffic.
3.  **Inject**: It listens on a Unix Socket (`/tmp/socket_bridge_control`) for external commands (from `bridge-ctl`) and injects them into the stream.
4.  **Loopback**: To keep the UI in sync, injected commands are sent BOTH to the Server (to affect Audio) and back to the Client (to update the UI).

## 2. The Protocol
The protocol is binary with a fixed header structure.

### Packet Structure
`[Header: 4B] [Length: 4B] [SessionID: 4B] [Index: 1B] [CommandStr] [Type: 1B] [Count: 1B] [Value]`

*   **Magic Header**: `0xF2B49E2C` (Little Endian: `2C 9E B4 F2`)
*   **Length**: Payload length (excluding Header/Length).
*   **Session ID**: 4 Bytes, dynamic (Sniffed from Client traffic).
*   **Index**: The target Fader ID + Base Offset.

### Commands

#### 1. Channel Output Mute
*   **Command**: `"channelOutputMute\0"`
*   **Base Offset**: `0x1C`
*   **Type**: `0x01`
*   **Value**: `0x01`
*   **Action**: `0x02` (Mute), `0x03` (Unmute)

#### 2. Channel Input Source
*   **Command**: `"channelInputSource\0"`
*   **Base Offset**: `0x1C`
*   **Type**: `0x05`
*   **Value**: `u32` (Source ID)
*   **Note**: Changing Source often requires a `inputMicrophoneType` sequence (`-1` then `4`) to prevent UI corruption.

#### 3. Fader Level
*   **Command**: `"faderLevel\0"`
*   **Base Offset**: `0x04`
*   **Type**: `0x05`
*   **Value**: `u32` (Level)
*   **Note**: Works for **Virtual Faders** only. Physical faders are read-only via this protocol.

## 3. Mappings

### Fader Indices
| Index | Type     | Protocol ID (Mute/Source) | Protocol ID (Level) |
| :--- | :--- | :--- | :--- |
| **0** | Physical 1 | `0x1C` | `0x04` |
| **1** | Physical 2 | `0x1D` | `0x05` |
| **2** | Physical 3 | `0x1E` | `0x06` |
| **3** | Physical 4 | `0x1F` | `0x07` |
| **4** | Physical 5 | `0x20` | `0x08` |
| **5** | Physical 6 | `0x21` | `0x09` |
| **6** | Virtual 1  | `0x22` | `0x0A` |
| **7** | Virtual 2  | `0x23` | `0x0B` |
| **8** | Virtual 3  | `0x24` | `0x0C` |

### Source IDs
*   **0-3**: Combo 1-4 (Mono)
*   **4-6**: Combo Stereo Pairs (1+2, 2+3, 3+4)
*   **7**: USB 1
*   **8**: USB 1 Chat
*   **9**: USB 2
*   **10**: Bluetooth
*   **11**: Soundpad
*   **12-15**: Virtual (Game, Music, A, B)
*   **16**: CallMe 1

## 4. Helper Tools
*   **`tcp-bridge`**: The main proxy binary.
*   **`bridge-ctl`**: CLI tool (`mute`, `source`, `level`, `mic_type`).
*   **`run-proxy.sh`**: Startup script (sets up IP alias, iptables, starts bridge).
