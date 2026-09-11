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

Early. The wire format is reverse-engineered and verified, the scaffold is in
place, the layers are landing one at a time. Not yet usable.

See [`docs/architecture.md`](docs/architecture.md) for the design and the
roadmap.

## Documentation

| Document | Contents |
|---|---|
| [`docs/architecture.md`](docs/architecture.md) | Design, layering, roadmap |
| [`docs/protocol.md`](docs/protocol.md) | Wire reference: SysEx, NRPN, dump layout |
| `cargo doc --open` | API reference |

## Supported hardware

DeepMind 6, 12 and 12D. All three share one model ID and one protocol.

Comms protocol versions 6 (as documented in the user manual) and 7 (shipping
firmware) are both handled.

## Development

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
cargo fmt --all --check
```

## License

Apache-2.0. See [LICENSE](LICENSE).

Not affiliated with or endorsed by Behringer or Music Tribe.
