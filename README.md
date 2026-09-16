<p align="center">
  <img src="https://raw.githubusercontent.com/MysteriousWolf/deepmind-midi/main/docs/banner.svg" alt="deepmind-midi" width="800">
</p>

<p align="center">
  <a href="https://crates.io/crates/deepmind-midi"><img src="https://img.shields.io/crates/v/deepmind-midi" alt="crates.io"></a>
  <a href="https://docs.rs/deepmind-midi"><img src="https://img.shields.io/docsrs/deepmind-midi" alt="docs.rs"></a>
  <a href="https://github.com/MysteriousWolf/deepmind-midi/actions/workflows/ci.yml"><img src="https://github.com/MysteriousWolf/deepmind-midi/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange" alt="Rust 1.85+">
  <img src="https://img.shields.io/badge/no__std-yes-blue" alt="no_std">
  <a href="https://github.com/MysteriousWolf/deepmind-midi/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-green" alt="License: Apache-2.0"></a>
</p>

Rust library for the Behringer DeepMind MIDI protocol. Sans-IO, `no_std`,
no dependencies.

The host program owns the MIDI connection and feeds bytes in and out; the
library turns them into typed messages, typed parameters and a tracked view of
the synthesizer.

```
DeepMind <--MIDI--> host program <--bytes--> deepmind-midi
```

```rust
let mut device: Device = Device::new(DeviceId::Unit(0));

device.feed(&bytes_from_port);                     // inbound, any chunking
device.tick(now_ms);                               // the host owns the clock
device.drain_tx(|bytes| port.send(bytes))?;        // outbound
while let Some(event) = device.poll_event() {}     // results

device.edit(|p| p.set_vcf_frequency(200))?;        // becomes one NRPN message
```

It also answers the questions an editor has to ask before it can draw anything:
what the instrument prints over each of its own faders, where an effect's
controls sit and what colour its panel is, how the four effect engines are
wired, and which parameters are read about a centre rather than from the bottom
of their range. `generator` goes one step further and publishes the shapes a
set of parameters makes — the envelope, the LFO wave, the filter response, the
arpeggiator's gates — as functions a host samples into its own screen, saying
in the type where an axis is published and where it is only an ordering. The `descriptions` feature adds a sentence per parameter and per
effect slot saying what the control does; leaving it off keeps that prose out of
an embedded build.

**Status: early.** Every layer is implemented and tested against the manual:
MIDI decoding, SysEx, the 242-parameter table, the 35 effect panels, the
instrument's own front panel, programs, `.syx` files, the device state machine,
a blocking transport adapter and a simulated synthesizer. None of it has been
run against real hardware yet.

There is no command-line tool and there will not be one. A host needs a MIDI
backend, and the library deliberately has no opinion about which.

## Hardware

DeepMind 6, 6X, 12, 12X, 12D and 12XD. One protocol across all six; they
differ only in voice count and whether there is a keyboard. Comms protocol
versions 6 and 7, firmware 1.0 and 1.1.

## Testing a host without a synthesizer

The `sim` feature is the other end of the conversation: it answers requests,
applies the edits it is sent and reports what it heard.

```rust
let mut synth: Synth = Synth::new(DeviceId::Unit(0), sound);
let mut host: Device = Device::new(DeviceId::Unit(0));

host.request_edit_buffer()?;
host.drain_tx(|bytes| { synth.feed(bytes); Ok(()) })?;
synth.drain_tx(|bytes| { host.feed(bytes); Ok(()) })?;

assert!(matches!(host.poll_event(), Some(Event::EditBuffer(_))));
```

A `syx::File` can stand in for the unit's memory, so a preset pack is a
simulated synthesizer. It has no clock: not draining it is a synthesizer that
has not replied yet, which is how a host's timeout path gets tested.

## Documentation

| | |
|---|---|
| [Protocol](https://github.com/MysteriousWolf/deepmind-midi/blob/main/docs/midi-spec.md) | Signal path, SysEx, NRPN, all 242 parameters, the CC map, value tables |
| [Effects](https://github.com/MysteriousWolf/deepmind-midi/blob/main/docs/effects.md) | All 35 algorithms: a drawing of each panel and what its twelve slots do, which the `effect` module also publishes, along with the grid and colours the drawings are made from |
| [Architecture](https://github.com/MysteriousWolf/deepmind-midi/blob/main/docs/architecture.md) | Design, layering, testing, releases, status |
| [`spec/`](https://github.com/MysteriousWolf/deepmind-midi/tree/main/spec) | The same protocol as TOML. Source of truth for the docs and the code |
| [NOTICE](https://github.com/MysteriousWolf/deepmind-midi/blob/main/NOTICE) | Where the descriptions and panel colours come from |
| [docs.rs](https://docs.rs/deepmind-midi) | API reference, or `cargo doc --open` |

## Development

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo +1.85.0 check --workspace --all-features   # the MSRV, which CI also checks
```

The rustdoc line is there because a broken or redundant intra-doc link is an
error in CI and nothing else catches one.

Documentation, diagrams, the effect panel drawings and the library's parameter
tables are generated from `spec/`. Edit the spec, then:

```sh
cargo xtask docs      # regenerate docs/
cargo xtask codegen   # regenerate the library's generated sources
```

Editing a spec file without regenerating fails `cargo test`.

Fuzzing needs nightly and `cargo-fuzz`:

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run frame -- -dict=fuzz/deepmind.dict   # or decoder, file, program, device
```

The factory preset packs are not in the repository. The tests that read them
are ignored unless `DEEPMIND_PACKS` points at your own copy:

```sh
DEEPMIND_PACKS=~/deepmind/packs cargo test -p deepmind-midi --test packs -- --include-ignored
```

## License

Apache-2.0. See [LICENSE](https://github.com/MysteriousWolf/deepmind-midi/blob/main/LICENSE).

Not affiliated with or endorsed by Behringer or Music Tribe.
