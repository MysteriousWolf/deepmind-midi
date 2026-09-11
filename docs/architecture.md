# Architecture

## Goal

The host program owns the MIDI connection. This library owns everything else:
framing, parameter semantics, device state, preset files. That split is the one
constraint everything else follows from.

```
DeepMind <--MIDI--> host program <--bytes--> deepmind-midi
                    (midir, alsa, coremidi,
                     a file, a test harness)
```

## The API a host uses

Nothing in a host program looks up an offset or a controller number. One object
holds the synthesizer; changing it is changing the synthesizer.

```rust
let mut synth = Synth::new(DeviceId::Unit(0));

// Read. Typed, grouped like the front panel.
let cutoff = synth.program().vcf.frequency;
println!("{cutoff}");                      // 1.2 kHz

// Write. Mutate; the library works out which messages that implies.
synth.edit(|p| {
    p.vcf.frequency = Frequency::hz(1200.0);
    p.lfo1.shape = LfoShape::Triangle;
    p.mod_matrix[0] = ModBus::new(ModSource::Lfo2, ModDest::VcfFreq, -40);
});
```

`edit` diffs the program against what the synthesizer is believed to hold and
queues NRPN messages for exactly the offsets that changed. Two edits to the same
parameter in one closure send one message.

Values are newtypes, not bare bytes:

```rust
p.vcf.frequency.raw()    // 0..=255, what goes on the wire
p.vcf.frequency.hz()     // 50.0..=20000.0, what the synthesizer shows
p.lfo1.shape             // an enum, not a magic number
```

Both the field layout and the conversions come from `spec/`, so the offset map
exists in one place and no host code repeats it.

## Sans-IO

No IO, no threads, no blocking, no clock. The host drives a state machine:

```rust
device.feed(&bytes_from_port);                    // inbound, any chunking
device.tick(now_ms);                              // caller supplies the clock
device.drain_tx(|bytes| port.send(bytes))?;       // outbound
while let Some(event) = device.poll_event() { }   // results
```

Why:

- **Testable.** A fake clock and a byte vector reproduce any timing bug exactly.
  Hardware-dependent tests are the ones that rot.
- **Portable.** No `std::time`, no executor. Works under `no_std`, in a JACK
  callback, in a tokio task, in WASM.
- **Honest.** MIDI transports vary wildly in latency and chunking. An async
  facade hides that rather than solving it.

`drain_tx` takes a closure, which covers most of what a callback API would,
without making `Device` generic over its IO. For hosts that want less
boilerplate, a `transport` feature adds a trait and a driver loop:

```rust
pub trait Transport {
    fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn send(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn now_ms(&self) -> u64;
}

device.run_blocking(&mut port, |event| { /* ... */ })?;
```

A trait and a loop, no executor and no channels. An async host writes its own
loop against the core in about thirty lines and keeps control of cancellation
and backpressure.

## Layers

```
ids       addressing: DeviceId, Model, Bank, ProgramNumber
error     one error type; every rejection carries the offending value

wire      MIDI bytes: message decode, running status, SysEx reassembly
sysex     DeepMind framing, packed MS-bit codec, typed Message enum
param     the parameter table: IDs, NRPN numbers, ranges, enums, formatting
program   Program, Global, Pattern, Bank: decode and encode parameter bytes
device    the sans-IO state machine: requests, timeouts, known state, events
syx       .syx files and preset packs
```

Each layer depends only on those above it. `wire` does not know what a DeepMind
is; `param` does not know what a message is.

## One table, three consumers

A parameter's NRPN number is its byte offset in a program dump. That one fact
collapses NRPN edits, dump parsing and dump building into a single table.

| File | Contents |
|---|---|
| `spec/parameters.toml` | 242 program parameters |
| `spec/enums.toml` | 28 value tables, including 132 modulation destinations |
| `spec/controllers.toml` | 112 MIDI controllers, 90 mapped to a parameter |
| `spec/messages.toml` | 22 SysEx messages |
| `spec/globals.toml` | 25 device-wide settings |
| `spec/effects.toml` | 370 effect parameters across 35 algorithms |

`cargo xtask docs` renders the tables and Mermaid diagrams in
[`midi-spec.md`](midi-spec.md), and writes the diagram sources to
`docs/diagrams/*.mmd`. The diagrams are embedded inline as well, from the same
strings, so nothing needs a build step to read.

The same files will generate the `Program` struct, its typed fields and its
conversions. A correction gets made once.

### Keeping generated output honest

The failure mode is silent, so it is caught twice:

- `generated_documentation_is_current` compares the checked-in document against a
  fresh render, so a stale checkout fails `cargo test`.
- `cargo xtask docs --check` does the same without writing, and names the command
  to run. CI runs it for the clearer error.

CI does not regenerate and commit. Auto-committing to contributor branches is
worse than a failure that says what to run.

The loader validates before rendering: offsets must cover 0..=241 exactly, every
referenced value table must exist, an enumerated parameter's maximum must match
its table, no controller number may repeat, and no two controllers may claim the
same parameter. Not theoretical. The enum check caught the parameter table
carrying firmware 1.0 ranges while the value tables carried firmware 1.1 lists.

Beyond that, correctness comes from disagreeing sources. The parameter table was
built from the manual, then checked against a MIDI Designer layout that had no
part in building it. All 35 of its named NRPN controls agree, including the three
offsets where this specification departs from the manual's printed names.

## State is a set of claims, not a cache

The synthesizer answers no per-parameter reads. Dumps can be requested; edits
can be sent and presumed to have landed. Plain values would conflate the two, so
every tracked value records where it came from:

```rust
pub enum Known<T> {
    Unknown,
    Assumed { value: T, sent_at: u64 },   // sent, not confirmed
    Confirmed { value: T, at: u64 },      // came back in a dump
}
```

The host decides whether to trust an assumed value or re-request the edit buffer
first. After sending edits, an edit buffer dump is the only way to resync, and
the library never polls on its own.

## Two kinds of version

Independent, and both matter.

**Comms protocol version**, stamped into every dump. Version 6 (documented) and
version 7 (shipping firmware) carry 242 and 245 bytes of program data. Offsets
0-241 are identical; 242-244 are reserved and preserved on round-trip.

**Firmware version**, read via device inquiry. Firmware 1.1 added modulation
sources and destinations and an FX algorithm, renumbering three value tables. A
program written on 1.1 using modulation source 23 means something else on 1.0.

The library reads firmware versions. It does not write firmware: updates are a
vendor SysEx file streamed over an undocumented bootloader protocol, and
blind-relaying that bricks synthesizers.

## Crate layout

```
spec/            the machine-readable specification
deepmind-midi/   the library
xtask/           generates the documentation from spec/
deepmind-cli/    a midir host: dump banks, import packs, monitor traffic
```

One library crate. A workspace split buys version churn and nothing else at this
size. Features cover the axes that matter: `std` (default), `alloc`, `serde`,
`transport`.

`deepmind-cli` exists to prove the library is pleasant to use against real
hardware. It is not part of the published surface.

## Testing

- **Property tests** on every codec. Round-trip is the invariant.
- **Fuzzing** on `feed()` and the SysEx parser. A MIDI port is untrusted input
  and the library must never panic on it.
- **Golden tests** against the factory preset packs.

The packs are not committed. Redistribution rights on Behringer sound banks are
unclear and a library repository is the wrong place to find out. The tests read a
path from an environment variable and skip when it is unset; the repository holds
a small synthetic fixture and expected checksums.

## Versioning and releases

`Cargo.toml` holds the version and nothing else does. The scheme is
`YY.RELEASE.PATCH`: `26.1.0` is the first release of 2026, `26.1.1` its first
patch, `26.2.0` the second release of the year.

A human owns it, editing one line by hand. CI fails when the version is not
ahead of the newest release tag, so the first pull request merged after a
release has to move it. That check is eight lines of shell in the workflow,
where anyone reading the workflow can see it.

Having the release workflow bump and commit instead would make the version in
the tree a lie between releases, and would need write access to the default
branch. This way the tree always states what it will release next and the
release job only reads.

Releasing is one click. The workflow verifies the build, reads the version,
refuses to proceed if that tag exists, then tags, releases and publishes. Notes
come from GitHub's generator, categorised by label through `.github/release.yml`,
with an optional opening paragraph from a small model on GitHub Models using the
built-in token; any failure there leaves the generated notes alone.
`CARGO_REGISTRY_TOKEN` is optional and a missing one warns rather than fails.

## Roadmap

| Step | Contents |
|---|---|
| 1 | Workspace, CI, lint policy, the specification, generation and release tooling |
| 2 | `spec/effects.toml`: per-effect parameter names |
| 3 | `wire` and `sysex`: framing, packed MS-bit codec, typed messages |
| 4 | `param`: code generated from `spec/`, typed values |
| 5 | `program`: the `Program` struct, decode and encode, `.syx` import and export |
| 6 | `device`: the state machine, events, timeouts, provenance, `edit` |
| 7 | `transport`: the blocking adapter |
| 8 | `deepmind-cli` |

### Next: the wire layer

Framing, the packed MS-bit codec and typed messages. `spec/` now describes every
byte a program carries, so the remaining steps are code.
