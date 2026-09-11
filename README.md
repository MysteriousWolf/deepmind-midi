# deepmind-midi

Rust library for the Behringer DeepMind MIDI protocol.

The library does no IO. The host program owns the MIDI connection and feeds
bytes in and out; the library turns them into typed messages, typed parameters
and a tracked view of the synthesizer.

```
DeepMind <--MIDI--> host program <--bytes--> deepmind-midi
```

**Status: early.** The protocol is reverse-engineered, verified and written
down. The code layers are landing one at a time: MIDI decoding, SysEx and the
parameter table are in; programs and device state are not. Not yet usable.

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

Documentation, diagrams, the effect panel drawings and the library's parameter
tables are generated from `spec/`:

```sh
cargo xtask docs               # regenerate docs/
cargo xtask docs --check       # fail if stale
cargo xtask codegen            # regenerate deepmind-midi/src/param/generated.rs
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
