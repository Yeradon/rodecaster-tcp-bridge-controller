# Rodecaster ValueTree Binary Protocol

## Message Structure

Each message starts with a fixed header:

```
┌──────────────────┬──────────────────┬─────────────────────┐
│ Prefix (4 bytes) │ Size (4 bytes)   │ Payload             │
│ 2C 9E B4 F2      │ Little-endian    │ Root node           │
└──────────────────┴──────────────────┴─────────────────────┘
```

- **Prefix**: Fixed magic bytes `2C 9E B4 F2`
- **Size**: 32-bit little-endian integer indicating payload size

## Node Structure

```
┌─────────────┬───────────────┬────────────────┬──────────────┬────────────────┬──────────────┐
│ Marker (?)  │ Name          │ Prop Count     │ Properties   │ Child Count    │ Children     │
│ 0x02 if has │ Null-term     │ Size + Count   │ ...          │ Size + Count   │ ...          │
│ children    │ string        │                │              │                │              │
└─────────────┴───────────────┴────────────────┴──────────────┴────────────────┴──────────────┘
```

- **Marker**: `0x02` byte present if node has children (absent otherwise)
- **Name**: Null-terminated UTF-8 string
- **Property Count**: Size byte (0, 1, or 2) + count value
- **Child Count**: Always present, Size byte + count value

## Count Encoding

```
Size Byte | Meaning
----------|----------------------------------
0x00      | Count = 0
0x01      | Count = next 1 byte (u8)
0x02      | Count = next 2 bytes (u16 LE)
```

## Property Structure

```
┌──────────────┬─────────────┬──────────────────┬─────────────────────────────────┐
│ Name         │ Size (1)    │ Length (1)       │ Type + Data                     │
│ Null-term    │ Size of len │ Total bytes      │ (length bytes)                  │
└──────────────┴─────────────┴──────────────────┴─────────────────────────────────┘
```

- **Name**: Null-terminated UTF-8 string
- **Size**: Size of the length field (typically 0x01)
- **Length**: Total bytes of type + data
- **Type + Data**: First byte is type indicator, rest is data

## Data Types

| Type | Bytes | Description                           | Example                      |
|------|-------|---------------------------------------|------------------------------|
| 0x01 | 4     | int32 (little-endian)                 | `01 45 00 00 00` → 69        |
| 0x02 | 0     | boolean true                          | `02` → true                  |
| 0x03 | 0     | boolean false                         | `03` → false                 |
| 0x04 | 8     | float64 (IEEE 754 LE)                 | `04 00 00 00 00 00 00 F0 3F` → 1.0 |
| 0x05 | var   | null-terminated string                | `05 64 65 00` → "de"         |
| 0x06 | 8     | int64 (little-endian)                 | `06 00 D7 BA 51 98 01 00 00` → timestamp |
| 0x08 | var   | binary blob                           | Base64 encoded in XML        |

## Example Hex Dump

```
0000: 2C 9E B4 F2 A4 89 01 00  │ Header: prefix + size (100772)
0008: 02 52 6F 64 65 63 61 73  │ 02 = has children, "Rodecas"
0010: 74 65 72 00              │ "ter\0"
0014: 00                       │ prop_count_size=0 (no properties)
0015: 02 6E 01                 │ child_count_size=2, count=0x016E (366)
0018: 50 48 59 53 49 43 41 4C  │ First child: "PHYSICAL"
...
```

## Parsed Structure Example

```xml
<Rodecaster>
  <PHYSICALINTERFACE onOffSw="1" hwScreenBrightness="250">
    <POT potMin="0" potMax="127" potLevel="69"/>
    <FADER faderMin="0" faderMax="127" faderLevel="68"/>
    <METER meterStereo="true" meterPeakL="0" meterLevelL="0"/>
    <PADBUTTON padButtonPressed="false" padButtonPreview="false"/>
  </PHYSICALINTERFACE>
  <SYSTEM systemFirmwareVersion="1.6.9" systemSerialNumber="IR0046451">
    <STORAGEVOLUME storageVolumeName="" storageVolumeCapacity="0"/>
  </SYSTEM>
  <NETWORK wifiSSID="Ultra8005" wifi="true"/>
  <CHANNEL channelInputSource="12" channelOutputMute="false"/>
  ...
</Rodecaster>
```

## Parser Implementation

See `startup-parser/src/main.rs` for a complete Rust implementation.

Usage:
```bash
cd startup-parser
cargo run --release -- ../startup.bin ../parsed.xml
```
