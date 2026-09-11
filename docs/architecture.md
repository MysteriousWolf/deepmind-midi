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

### The transport adapter

Writing that loop by hand for every host is boilerplate, so a `transport`
feature adds a thin adapter over the same core:

```rust
pub trait Transport {
    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn send(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn now_ms(&self) -> u64;
}

device.run_blocking(&mut port, |event| { /* ... */ })?;
```

That is the whole addition: a trait and a driver loop. No executor, no channels,
no threads. An async host writes its own loop against the sans-IO core, which is
about thirty lines and lets it own cancellation and backpressure properly. If a
tokio wrapper turns out to be worth it later it can be added as another feature,
but it will never be the only way in.

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

`spec/` is the source of truth, and it is already in place:

| File | Contents |
|---|---|
| `spec/parameters.toml` | 242 program parameters |
| `spec/enums.toml` | 27 value tables, including the 132 modulation destinations |
| `spec/messages.toml` | 22 SysEx messages |
| `spec/globals.toml` | 24 device-wide settings |

`cargo xtask docs` renders the reference tables in `docs/midi-spec.md`. The same
files will generate the `Param` enum, its lookup tables, and typed accessors on
`Program` when that layer lands. A correction is made once, in one file.

### Keeping generated output honest

Three layers, because the failure mode is silent:

- `cargo xtask docs` regenerates in place.
- `cargo xtask docs --check` reports staleness without writing. CI runs it.
- `generated_documentation_is_current` runs the same comparison as a test, so a
  stale checkout fails `cargo test` whether or not anyone remembers the check.

`cargo xtask hooks install` writes a pre-commit hook that regenerates and stages
the document, so a commit cannot carry a stale one in the first place. It is
opt-in: CI auto-committing to contributor branches is worse than a clear failure
that names the command to run.

The loader validates the spec before rendering anything. Offsets must cover
0..=241 exactly, every referenced value table must exist, and an enumerated
parameter's maximum must match its table. That last check is not theoretical: it
caught the parameter table carrying firmware 1.0 ranges while the value tables
carried firmware 1.1 lists.

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

## Two kinds of version

They are independent and both matter.

**Comms protocol version**, stamped into every dump. Version 6 (documented) and
version 7 (shipping firmware, both factory packs) differ in program data length,
242 versus 245 bytes. Offsets 0-241 are identical; 242-244 are reserved and
preserved verbatim on round-trip.

**Firmware version**, read via device inquiry. Firmware 1.1 added modulation
sources and destinations and an FX algorithm, renumbering three value tables in
the process. A program written on 1.1 that uses modulation source 23 means
something different on 1.0.

The library reads firmware versions and exposes them. It does not write firmware:
updates are a vendor SysEx file streamed by Behringer's own updater over an
undocumented bootloader protocol, and blind-relaying that is a good way to brick
a synthesizer.

## Crate layout

```
spec/            the machine-readable protocol specification
deepmind-midi/   the library
xtask/           documentation generation, drift checks, release version bumps
deepmind-cli/    a midir host: dump banks, import packs, monitor traffic
```

One library crate, not a workspace of five. A split buys version churn and buys
nothing at this size. Feature flags handle the axes that matter:

- `std` (default), `alloc`, `serde`, `transport`

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

- `cargo doc` is the function-level reference. `missing_docs` is a warning that
  CI treats as an error, so a public item without documentation fails the build.
- `docs/midi-spec.md` is the wire reference: hand-written prose around generated
  tables, verified against real dumps.
- This file covers the design.

## Versioning and releases

`Cargo.toml` holds the version and nothing else does. The scheme is
`YY.RELEASE.PATCH`: `26.1.0` is the first release of 2026, `26.1.1` its first
patch, `26.2.0` the second release of the year. The year rolls over on its own.

Releasing is one click in the Actions tab. The workflow verifies the build,
bumps the version, tags, writes release notes, and publishes. Both of its
secrets are optional: without `CARGO_REGISTRY_TOKEN` it skips the crates.io
publish, without `ANTHROPIC_API_KEY` it ships GitHub's generated notes without a
prose summary on top. Either way it warns and carries on rather than failing
halfway through a release.

## Roadmap

| Step | Contents |
|---|---|
| 1 | Workspace, CI, lint policy, the specification, generation and release tooling |
| 2 | `wire` and `sysex`: framing, packed MS-bit codec, typed messages |
| 3 | `param`: code generated from `spec/`, typed values |
| 4 | `program`: decode and encode, `.syx` import and export |
| 5 | `device`: the state machine, events, timeouts, provenance |
| 6 | `transport`: the blocking adapter |
| 7 | `deepmind-cli` |
| 8 | Per-effect parameter names, from section 9 of the manual |
