# DeepMind MIDI protocol reference

Everything here is taken from the DeepMind 12 user manual (sections 17-19) and
verified byte for byte against the Behringer *Retro Electro* and *Synth Wizards*
factory preset packs. Where the manual and the hardware disagree, the hardware
wins and the difference is called out.

## 1. Addressing

| Field | Value |
|---|---|
| Manufacturer ID | `00 20 32` (Behringer) |
| Model ID | `20` (all DeepMind variants) |
| Device ID | `00`-`0F`, or `7F` to broadcast |

The device ID doubles as the synthesizer's global MIDI channel. A unit set to
channel 1 answers SysEx addressed to device ID `00`.

Factory preset packs are not consistent about this: *Retro Electro* ships with
`7F`, *Synth Wizards* with `00`. A parser must accept both.

## 2. Packed MS-bit encoding

Bulk dumps carry 8-bit data over 7-bit SysEx using "packed MS bit" format. Each
group of 7 raw bytes becomes 8 transmitted bytes: one byte holding the seven
high bits, then the seven payloads with their high bit cleared.

```
raw:     d0 d1 d2 d3 d4 d5 d6                   (7 bytes, 8 bits each)
packed:  M  d0 d1 d2 d3 d4 d5 d6                (8 bytes, 7 bits each)
         ^
         bit n of M is bit 7 of raw byte n
```

So `packed_len = ceil(raw_len / 7) * 8`, and the final group is short when the
raw length is not a multiple of 7.

## 3. Message types

Request and response command bytes, in the position immediately after the device
ID.

| Cmd | Message | Payload |
|---|---|---|
| `01` | Program dump request | bank, program |
| `02` | Program dump response | version, bank, program, packed program data |
| `03` | Edit buffer dump request | - |
| `04` | Edit buffer dump response | version, packed program data |
| `05` | Global parameter dump request | - |
| `06` | Global parameter dump response | version, 45 raw bytes in 56 packed |
| `07` | Single user pattern dump request | pattern (0-31) |
| `08` | Single user pattern dump response | version, pattern, 65 raw bytes in 80 packed |
| `09` | Program bank dump request | bank, first program, last program |
| `0A` | Bank program names dump request | bank |
| `0B` | Bank program names dump response | version, bank, 2048 raw bytes in 2344 packed |
| `0C` | Single program name dump request | bank, program |
| `0D` | Single program name dump response | version, bank, 16 raw bytes in 24 packed |
| `0E` | Edit buffer pattern dump request | - |
| `08` | Edit buffer pattern dump response | version, 65 raw bytes in 80 packed |
| `11` | Calibration data dump request | - |
| `12` | Calibration data dump response | version, 688 packed bytes |
| `1B` | Chord memory dump request | - |
| `1C` | Chord memory dump response | version, 26 raw bytes in 32 packed |
| `1D` | Poly chord memory dump request | - |
| `1E` | Poly chord memory dump response | version, 512 raw bytes in 592 packed |
| `00` | Control app notify request | one reserved byte |
| `10` | Control app notify response | rx channel, tx channel, interface, bank, program |

Two quirks in that table are the manual's, not typos here:

- Command `08` is listed for both the single user pattern response and the edit
  buffer pattern response. They are told apart by length: the single pattern
  response carries a pattern number byte before the payload.
- A bank dump request produces a sequence of individual `02` program dump
  responses, not one aggregate message.

### Device inquiry

The DeepMind also answers the universal non-realtime identity request, which is
the only way to discover a unit whose device ID you do not already know.

```
request:  F0 7E <dev|7F> 06 01 F7
response: F0 7E <dev> 06 02 00 20 32 20 00 01 00 <jn> 00 <ii> <nn> F7
```

`jn` packs the main software revision, `ii`/`nn` are the voice software major and
minor versions.

### Control app notify

Sending a `00` notify request tells the synthesizer that a control application is
attached to that interface. The synthesizer then enables NRPN and SysEx on it and
starts emitting extra traffic such as key-down and voice-allocation updates. This
is how the official editor gets live feedback, and it has to be sent explicitly.

The DeepMind keeps NRPN state *per interface*: MIDI DIN, USB and Wi-Fi each have
their own selected-parameter register.

## 4. Program data layout

This is the important find. The 14-bit NRPN number of a parameter **is** its byte
offset in the unpacked program dump. One parameter, one byte, no packing tricks
beyond the MS-bit encoding. The manual's NRPN table therefore doubles as the dump
layout, which means one table drives NRPN edits, dump parsing and dump building.

| Offset | Field |
|---|---|
| 0-6 | LFO 1 |
| 7-13 | LFO 2 |
| 14-38 | Oscillators, noise, portamento, pitch bend |
| 39-52 | VCF |
| 53-61 | VCA envelope |
| 62-70 | VCF envelope |
| 71-79 | Mod envelope |
| 80-92 | VCA, voicing, drift |
| 93-116 | Modulation matrix, 8 slots of source/destination/depth |
| 117-154 | Control sequencer |
| 155-164 | Arpeggiator |
| 165-222 | FX routing, 4 slots of type plus 12 parameters, output gains, mode |
| 223-239 | Program name, 16 ASCII chars plus terminator |
| 240 | Program category |
| 241 | Program transpose, 80-176, with 128 meaning no transpose |

### Protocol version 6 versus 7

The manual documents version 6: 242 raw bytes packed into 278. Current firmware
and both factory packs emit version 7: **245 raw bytes packed into 280**, total
message length 291.

Offsets 0-241 are identical between the two versions. The three extra bytes sit
at 242-244 and are zero in all 256 factory programs examined. Treat them as
reserved, preserve them on round-trip, and do not assume a dump is 242 bytes.

Sanity checks used to confirm the mapping against real data:

- offset 241 is `0x80` (128, no transpose) in 233 of 256 factory programs
- offsets 219-221 (FX output gains) cluster hard on 100, within the documented
  0-150 range
- offset 222 (FX mode) only ever holds 0 or 1, within the documented 0-2 range
- offsets 223-239 decode as clean ASCII names in every program

## 5. NRPN edits

```
B0 63 <msb>   NRPN parameter MSB
B0 62 <lsb>   NRPN parameter LSB
B0 06 <msb>   data entry MSB
B0 26 <lsb>   data entry LSB
```

With running status this collapses to `B0 63 aa 62 bb 06 cc 26 dd`.

Two behaviours worth building around:

- The data entry MSB is optional for switches and for any parameter whose range
  fits in 0-127.
- Once a parameter is selected, repeated data entry messages keep applying to it.
  A host sweeping a single control should send the selection once and then only
  data entry pairs.

Parameter ranges go up to 0-255, so a value does not fit in a single 7-bit data
byte. The 14-bit data entry pair carries it.

## 6. Reading state

The synthesizer does not answer per-parameter reads. There is no "what is LFO 1
rate right now" message. State comes from dumps only:

- edit buffer dump for what is currently being played
- program dump for a stored slot
- global parameter dump for device settings
- bank names dump for a cheap 128-program index without pulling full programs

A host that sends an NRPN edit and wants confirmation has to re-request the edit
buffer. This asymmetry drives the state model described in `architecture.md`.

## 7. Sources

- DeepMind 12 user manual, sections 16 through 19
- Behringer *Retro Electro* sound bank, 128 programs, protocol version 7
- Behringer *Synth Wizards* sound bank, 128 programs, protocol version 7
