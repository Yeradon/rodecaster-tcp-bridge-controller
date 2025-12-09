# Rodecaster Pro II: Protocol & Bridge Documentation

## Overview
This project successfully reverse-engineered the internal communication channel between the **UI** (`rc_gui_juce`) and the **Audio Engine** (`rc_audio_mixer`) on the Rodecaster Pro II. By injecting packets into this channel, we can programmatically control faders, mutes, and other mixer functions.

## 1. The Protocol
The communication happens over a Unix Domain Socket pair (FD 10 in `rc_gui_juce`). The protocol is binary with a fixed header.

### Packet Structure
`[Header: 4B] [Length: 4B] [Payload: Variable]`

*   **Magic Header**: `0xF2B49E2C` (Little Endian)
*   **Command Prefix**: `01 01 01 01` (Standard for control commands)

### Protocol Discovery

#### Protocol : Virtual Faders (1-3)
Used for the virtual sliders. The **Channel ID** is embedded in the *command string prefix*.
*   **Structure**: `[Prefix] [FaderID_Char] "channelOutputMute\0" [Type] [Val] [Action]`
*   **Fader IDs** (Prefix Character):
    *   Fader 1: `"` (0x22)
    *   Fader 2: `#` (0x23)
    *   Fader 3: `$` (0x24)
*   **Action** (Last Byte):
    *   Mute: `0x02`
    *   Unmute: `0x03`


---

## 2. The Socket Bridge (`socket-bridge`)
To inject these packets, we built a Rust tool that uses `ptrace` to intercept the target process (`rc_gui_juce`). It logs all the messages using formatted hex output and interprets known commands and formats them in a human-readable way.

### Robustness Features
*   **Safe Injection Point**: The bridge verifies that the instruction at `PC-4` is `SVC` (Syscall) before injecting. This prevents memory corruption by ensuring we only inject when the process is waiting at a syscall boundary.
*   **Clean Detach**: To prevent the "Zombie Freeze" (where the UI hangs after the tool exits), we use a strict sequence:
    1.  `SIGSTOP` all threads.
    2.  Wait for stop.
    3.  `ptrace::detach`.
    4.  `SIGCONT` all threads.

### Test Bridge (`test_bridge`)
A simple tool to inject packets into the socket bridge. It supports known commands in a human-readable way and a raw command to inject hex data.

### Run Script (`run-bridge.sh`)
A simple script to run the bridge.


### Deployment Script (`deploy.ps1`)
A simple script to build and upload the bridge to the device.
---

## 3. UI Synchronization
Injecting the *Output* command (`channelOutputMute`) mutes the audio but leaves the UI showing "Unmuted". This is a "Split Brain" state.

**Solution**:
No soloution found yet

## 4. Usage

### Deployment
Use the provided PowerShell script to build and upload:
```powershell
.\deploy.ps1
```

### Running the Bridge
On the device:
```bash
/tmp/run-bridge.sh
```

### Controlling (Examples)
From the device (or via UDP from PC):

**Mute Fader 1 (Physical):**
```bash
/tmp/test_bridge physical 22 1
```

**Simulate Touch (UI Sync):**
```bash
/tmp/test_bridge raw 2c9eb4f216000000010101010773637265656e546f756368656400010102
```
