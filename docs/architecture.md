# Architecture

## Goal

A host program owns a MIDI connection to a DeepMind. This library owns everything
else: framing, parameter semantics, device state, preset files. The split is
absolute, and it is the single most important constraint in the design.

```
DeepMind <--MIDI--> host program <--bytes--> deepmind-midi
                    (midir, alsa,
                     coremidi, a file,
                     a test harness)
```

## Sans-IO

The library performs no IO, spawns no threads, does not block and does not depend
on a clock. It exposes a state machine the host drives:

```rust
let mut device = Device::new(DeviceId::Unit(0));

// inbound: any chunking, including split SysEx
device.feed(&bytes_from_port);

// time: caller supplies a monotonic millisecond counter
device.tick(now_ms);

// outbound: drain and send
while let Some(message) = device.poll_tx() {
    port.send(message)?;
}

// results: drain and react
while let Some(event) = device.poll_event() {
    // ...
}
```

Why this shape:

- **Testable.** A fake clock and a byte vector reproduce any timing bug exactly.
  Hardware-dependent tests are the ones that rot.
- **Portable.** No `std::time`, no executor. Works under `no_std`, in a JACK
  callback, in a tokio task, in a WASM build.
- **Honest.** MIDI transports vary wildly in latency and chunking. Pretending
  otherwise behind an async facade hides the problems rather than solving them.

An optional async wrapper over channels can be added later as a feature. The
sans-IO core stays the only implementation.

## Layers

```
ids       addressing primitives: DeviceId, Model, Bank, ProgramNumber
error     one error type, every rejection carries the offending value

wire      MIDI byte level: message decode, running status, SysEx reassembly
sysex     DeepMind framing, packed MS-bit codec, typed Message enum
param     the parameter table: IDs, NRPN numbers, ranges, enums, formatting
program   Program, Global, Pattern, Bank: decode and encode raw parameter bytes
device    the sans-IO state machine: requests, timeouts, known state, events
syx       .syx file and preset pack import and export
```

Each layer depends only on the ones above it. `wire` does not know what a
DeepMind is. `param` does not know what a message is.

## One table, three consumers

The NRPN number of a parameter is its byte offset in a program dump (see
`protocol.md`). That single fact collapses what would otherwise be three separate
mappings into one.

`params/deepmind.toml` is the source of truth. Every parameter gets an entry:

```toml
[[param]]
id      = "lfo1_shape"
name    = "LFO 1 Shape"
group   = "lfo1"
nrpn    = 2
range   = { kind = "enum", values = [
    "Sine", "Triangle", "Square", "RampUp", "RampDown", "SampleHold", "SampleGlide",
] }
```

`cargo xtask codegen` turns that into:

- the `Param` enum and its lookup tables
- `docs/parameters.md`, a browsable reference
- typed accessors on `Program`

CI runs `cargo xtask docs --check` so generated output cannot drift from the
table. Adding a parameter means editing one TOML entry, not four Rust files.

## State is not a cache, it is a set of claims

The DeepMind answers no per-parameter reads. You can request dumps, and you can
send NRPN edits and *believe* they landed. A struct of plain values would quietly
lie about which is which, so every tracked value records its provenance:

```rust
pub enum Known<T> {
    /// Never observed.
    Unknown,
    /// We sent this value. The synthesizer has not confirmed it.
    Assumed { value: T, sent_at: u64 },
    /// This came back in a dump.
    Confirmed { value: T, at: u64 },
}
```

A host can then decide for itself whether to trust an assumed value or re-request
the edit buffer first. This is the difference between a library that works on one
desk and one that works on a stage.

Consequence worth stating plainly: after sending edits, the only way to resync is
an edit buffer dump request. The library will not silently poll for you.

## Protocol versions

Dumps carry a comms protocol version byte. Version 6 (documented) and version 7
(shipping firmware, both factory packs) differ in program data length, 242 versus
245 bytes. Offsets 0-241 are identical; 242-244 are reserved and preserved
verbatim on round-trip.

The parameter table gates entries by version range, so a v6 dump and a v7 dump
both decode into the same `Program` type without the caller caring.

## Crate layout

```
deepmind-midi/   the library
xtask/           codegen and documentation drift checks
deepmind-cli/    a midir host: dump banks, import packs, monitor traffic
```

One library crate, not a workspace of five. A split buys version churn and buys
nothing at this size. Feature flags handle the axes that matter:

- `std` (default), `alloc`, `serde`

`deepmind-cli` exists to prove the library is pleasant to use against real
hardware. It is not part of the published library surface.

## Testing

- **Property tests** on every codec: pack/unpack, message encode/decode, program
  encode/decode. Round-trip is the invariant.
- **Fuzzing** on `feed()` and the SysEx parser. A MIDI port is untrusted input and
  the library must never panic on it.
- **Golden tests** against the factory preset packs.

On those packs: they are not committed. Redistribution rights on Behringer sound
banks are unclear, and a library repository is the wrong place to find out. The
tests read a path from an environment variable and skip when it is unset, and the
repository holds a small synthetic fixture plus expected checksums.

## Documentation

- `cargo doc` is the function-level reference, published to GitHub Pages on
  merge. `missing_docs` is a warning that CI treats as an error.
- `docs/protocol.md` is the wire reference, written from the manual and verified
  against real dumps.
- `docs/parameters.md` is generated from the parameter table.
- This file covers the design.

## Roadmap

| Step | Contents |
|---|---|
| 1 | Workspace, CI, lint policy, these documents |
| 2 | `wire` and `sysex`: framing, packed MS-bit codec, typed messages |
| 3 | `param`: the table, codegen, generated reference |
| 4 | `program`: decode and encode, `.syx` import and export |
| 5 | `device`: the state machine, events, timeouts, provenance |
| 6 | `deepmind-cli` |
