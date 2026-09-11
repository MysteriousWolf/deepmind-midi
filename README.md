# deepmind-midi

Sans-IO Rust library for the Behringer DeepMind MIDI protocol: SysEx, NRPN,
programs, banks and device state.

The library does no IO. Your program owns the MIDI connection and feeds bytes in
and out; the library turns them into typed messages, typed parameters and a
tracked view of the synthesizer.

```
DeepMind <--MIDI--> your program <--bytes--> deepmind-midi
```

## Status

Early. The protocol is reverse-engineered, verified and written down; the code
layers are landing one at a time. Not yet usable.

## Documentation

| Document | Contents |
|---|---|
| [`docs/midi-spec.md`](docs/midi-spec.md) | The protocol: SysEx, NRPN, all 242 parameters, value tables |
| [`docs/architecture.md`](docs/architecture.md) | Design, layering, roadmap |
| `cargo doc --open` | API reference |

`spec/*.toml` is the machine-readable source of truth. The reference tables in
`docs/midi-spec.md` are generated from it, and the library's parameter tables
will be too.

## Supported hardware

DeepMind 6, 12 and 12D. All three share one model ID and one protocol.

Comms protocol versions 6 (documented) and 7 (shipping firmware) are both
handled, as are the value-table differences between firmware 1.0 and 1.1.

## Development

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
cargo fmt --all --check
```

Documentation is generated from `spec/`:

```sh
cargo xtask docs            # regenerate docs/midi-spec.md
cargo xtask docs --check    # fail if it is stale
cargo xtask hooks install   # regenerate and stage on every commit
```

Editing a spec file without regenerating fails `cargo test`.

## Releasing

Actions tab, Release workflow, pick `patch` or `release`. Versions are
`YY.RELEASE.PATCH`, so `26.1.0` is the first release of 2026 and `26.1.1` its
first patch. `Cargo.toml` is the only place the version lives.

## License

Apache-2.0. See [LICENSE](LICENSE).

Not affiliated with or endorsed by Behringer or Music Tribe.
