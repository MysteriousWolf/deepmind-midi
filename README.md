# deepmind-midi

Rust library for the Behringer DeepMind MIDI protocol.

The library does no IO. The host program owns the MIDI connection and feeds
bytes in and out; the library turns them into typed messages, typed parameters
and a tracked view of the synthesizer.

```
DeepMind <--MIDI--> host program <--bytes--> deepmind-midi
```

**Status: early.** The protocol is reverse-engineered, verified against the
manual and written down. Every layer is in: MIDI decoding, SysEx, the parameter
table, programs, `.syx` files, the device state machine and the blocking
transport adapter. Enough to drive a synthesizer from a host that owns the port,
and not yet run against one - which is the gap that matters, and the one thing
no amount of code here closes.

There is no command-line tool and there will not be one. A host needs a MIDI
backend, and refusing to have an opinion about which is the point of the
library.

## Testing a host without a synthesizer

The `sim` feature is the other end of the conversation: something that answers
requests, applies the edits it is sent and says what it heard, so a host can be
driven through a whole exchange with nothing plugged in.

```rust
let mut synth: Synth = Synth::new(DeviceId::Unit(0), sound);
let mut host: Device = Device::new(DeviceId::Unit(0));

host.request_edit_buffer()?;
host.drain_tx(|bytes| { synth.feed(bytes); Ok(()) })?;
synth.drain_tx(|bytes| { host.feed(bytes); Ok(()) })?;

assert!(matches!(host.poll_event(), Some(Event::EditBuffer(_))));
```

Stored programs come from a `Library`, which `syx::File` implements, so a preset
pack is the contents of a simulated unit. It has no clock: not draining it is a
synthesizer that has not replied yet, which is how a host's timeout path gets
tested.

It is built from the same specification as the rest of this crate, so it will
never find out that the manual is wrong. What it finds is a host that drives the
protocol wrongly, which is a different and more common bug.

## Documentation

| | |
|---|---|
| [Protocol](docs/midi-spec.md) | Signal path, SysEx, NRPN, all 242 parameters, the CC map, value tables |
| [Effects](docs/effects.md) | All 35 algorithms: a drawing of each panel and what its twelve slots do |
| [Architecture](docs/architecture.md) | Design, layering, roadmap |
| [`spec/`](spec/) | The same protocol as TOML. Source of truth for the docs and the code |
| [NOTICE](NOTICE) | Where the descriptions and panel colours come from |
| `cargo doc --open` | API reference |

## Hardware

DeepMind 6, 6X, 12, 12X, 12D and 12XD. One protocol across all six; they differ
only in voice count and whether there is a keyboard.

Handles comms protocol versions 6 and 7, and the value-table differences between
firmware 1.0 and 1.1.

## Development

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
cargo fmt --all --check
cargo +1.85.0 check --workspace --all-features   # the MSRV, which CI also checks
```

A current toolchain accepts things 1.85 does not, so the last line is worth
running before pushing.

Fuzzing needs nightly and `cargo-fuzz`, and lives in its own workspace under
`fuzz/`:

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run frame -- -dict=fuzz/deepmind.dict   # or decoder, file, program
```

The dictionary is worth the flag: without it the mutator spends its time
guessing at a five-byte SysEx header instead of at payloads.

The factory preset packs are not in the repository, so the tests that read them
skip unless you point them at your own copy:

```sh
DEEPMIND_PACKS=/path/to/presets cargo test -p deepmind-midi --test packs
```

Documentation, diagrams, the effect panel drawings and the library's parameter
tables are generated from `spec/`:

```sh
cargo xtask docs               # regenerate docs/
cargo xtask docs --check       # fail if stale
cargo xtask codegen            # regenerate the library's generated sources
cargo xtask codegen --check    # fail if stale
```

Editing a spec file without regenerating fails `cargo test`.

## Releasing

Versions are `YY.RELEASE.PATCH`: `26.1.0` is the first release of 2026, `26.1.1`
its first patch, `26.2.0` the second release of the year.

Edit `version` in `Cargo.toml`, in a pull request. CI fails any pull request
whose version is not ahead of the newest release tag, so the first one merged
after a release has to move it. Then run the Release workflow from the Actions
tab; it tags and publishes what `Cargo.toml` holds.

## License

Apache-2.0. See [LICENSE](LICENSE).

Not affiliated with or endorsed by Behringer or Music Tribe.
