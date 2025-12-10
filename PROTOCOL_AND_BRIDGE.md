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

## 4. Mix Link/Unlink Commands

Commands for controlling source linking in output mixes (Headphones, USB, etc.).

### Prefix Calculation
The prefix byte is calculated using a matrix formula:
```
prefix = source_index * 13 + mix_index
```

### Mix Commands

#### 1. Mix Link Request
*   **Command**: `"mixLinkRequest\0"`
*   **Prefix**: Calculated from formula
*   **Payload**: `01 07 08 01 01 02 01 01 02` (fixed values)

#### 2. Mix Unlink Request
*   **Command**: `"mixUnlinkRequest\0"`
*   **Prefix**: Calculated from formula
*   **Payload**: Same as Link

#### 3. Mix Disabled
*   **Command**: `"mixDisabled\0"`
*   **Prefix**: Calculated from formula
*   **Payload**: `01 01 <state>` (02=active, 03=disabled)

### Mix Indices (Output Buses) - Complete Mapping
| Index | Output |
| :--- | :--- |
| **10** | Headphone 1 |
| **11** | Headphone 2 |
| **12** | Headphone 3 |
| **13** | Headphone 4 |
| **14** | Speaker |
| **15** | Recording |
| **16** | Bluetooth |
| **17** | USB 1 |
| **18** | Chat |
| **19** | USB 2 |
| **20** | CallMe 1 |
| **21** | CallMe 2 |
| **22** | CallMe 3 |

### Source Indices (for Mix Commands) - Complete Mapping
| Index | Source |
| :--- | :--- |
| **4** | Combo 1 |
| **5** | Combo 2 |
| **6** | Combo 3 |
| **7** | Combo 4 |
| **8** | Combo 1+2 (Stereo) |
| **9** | Combo 2+3 (Stereo) |
| **10** | Combo 3+4 (Stereo) |
| **11** | USB 1 |
| **12** | Chat/CallMe |
| **13** | USB 2 |
| **14** | Bluetooth |
| **15** | SoundPad |

### CallMe Sources (Special Encoding)
CallMe sources use a different packet structure:
*   **Session ID**: `01 01 01 02` (hardcoded, different from regular `01 01 01 01`)
*   **Prefix**: 2 bytes instead of 1
    *   First byte: `4 + mix_index`
    *   Second byte: `callme_index` (1, 2, or 3)
*   **Packet size**: 40 bytes (vs 39 for regular sources)

**Example:** CallMe 1 in HP1 (mix 10): prefix = `0e 01` (14, 1)

## 5. Helper Tools
*   **`tcp-bridge`**: The main proxy binary.
*   **`bridge-ctl`**: CLI tool with commands:
    *   `mute`, `source`, `level`, `mic-type`, `touch`
    *   `mix-link`, `mix-unlink`, `mix-disable`
    *   `call-me-link`, `call-me-unlink`
*   **`run-proxy.sh`**: Startup script (sets up IP alias, iptables, starts bridge).

