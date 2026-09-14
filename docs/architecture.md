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
let mut device: Device = Device::new(DeviceId::Unit(0));

// Read. Named the way the synthesizer's own display names it, and carrying
// where the answer came from.
if let Some(program) = device.program().value() {
    println!("{}", program.name());         // Bass Sweep
    let shape = program.lfo1_shape();       // Some(LfoShape::Triangle)
}

// Write. Mutate; the library works out which messages that implies.
device.edit(|p| {
    p.set_vcf_frequency(200);
    p.set_lfo1_shape(LfoShape::Triangle);
    p.set_mod1_source(ModSource::Lfo2);
})?;
```

`edit` diffs the program against what the synthesizer is believed to hold and
queues NRPN messages for exactly the offsets that changed - `Program::changes`
is that diff, and `drain_tx` is what hands it to the port. Two edits to the same
parameter in one closure send one message, and an edit that puts a parameter
back where it was sends none.

A parameter with a value table reads as its table, and a switch as a `bool`:

```rust
program.get(ParamId::Lfo1Shape)   // 1, the byte in the dump
program.lfo1_shape()              // Some(LfoShape::Triangle)
program.lfo1_key_sync()           // false
```

What is not there is a conversion from the raw value to the number the
synthesizer displays, for the reason [Raw values stay raw until
measured](#raw-values-stay-raw-until-measured) gives. `program.vcf_frequency()`
is a byte, not a frequency, until the curve behind that byte is known.

The accessors and the value types are generated from `spec/`, so the offset map
exists in one place and no host code repeats it.

## A program is its bytes

A `Program` holds the dump it arrived as: 242 bytes, or 245 under comms protocol
version 7. Reading a parameter is a lookup and writing one is a byte, so a dump
that goes in comes out byte for byte, and the tests say so over every byte
pattern.

The alternative - a struct of 242 decoded fields - loses on all three counts
that matter here. A value no table lists could not be held, and hardware is
entitled to send one. The reserved bytes of a version 7 dump would have to be
carried alongside anyway. And three value tables were renumbered by firmware, so
a decoded field would have to pick a firmware at decode time, when nothing in a
stored program says which firmware wrote it.

So decoding is a view over the bytes rather than a copy of them. A typed
accessor answers `Option` where the byte is not a value its table names; the byte
is still there, and `Program::get` returns it. `Program::invalid` lists the
parameters holding something they do not accept, rather than a decoder refusing
the dump over one byte.

## Which value tables become a type

A value table becomes a Rust enum when its entries are a closed set of names:
`LfoShape::Triangle`, `ModSource::Lfo2`, `FxType::RotarySpkr`. Twenty-two of the
twenty-seven are.

The other five stay raw bytes with a label, because a type would be a worse way
of writing what they hold:

- The three clock divider tables list `1/2`, `3/8`, `1/16`. Those are divisions
  of a bar, not names, and `Div1Over2` says less than `1/2` does.
- `LFO Mono Mode` and `Arpeggiator Pattern` list the first of a run - `SPREAD-1`
  stands for `SPREAD-1` through `SPREAD-254` - so an enum of what is listed would
  answer `None` for almost every value the synthesizer sends.

Which of the two a table is, is read off the table rather than declared in it.
`spec/` describes the synthesizer; how that lands in Rust is the generator's
business, and putting the rule in the generator keeps the specification about the
hardware.

For the three tables firmware renumbered, a variant is what the value *means* and
`raw_for` is the byte it travels as on a given firmware. `ModSource::Expression`
is 6 on firmware 1.1 and does not exist on 1.0; `ModSource::Lfo1` is 7 on 1.1 and
6 on 1.0. The join between the two firmware tables is the entry's name, which is
also why `NoteOff Vel` and `Note Off Vel` are one variant: firmware respelled the
name of a value it kept.

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
program   Program: the dump's bytes, typed accessors, names, value types
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
| `spec/effects.toml` | 371 effect parameters across 35 algorithms |
| `spec/panels.toml` | how those 371 slots present themselves |
| `spec/layout.toml` | where each slot sits on the FX page, and the panel colours |
| `spec/mapping.toml` | how an address and a value become bytes |
| `spec/routing.toml` | how the four FX engines can be wired together |
| `spec/measurements.toml` | 29 raw values with what the synthesizer displayed |
| `spec/firmware.toml` | firmware versions that change the protocol |

`cargo xtask docs` renders the tables and Mermaid diagrams in
[`midi-spec.md`](midi-spec.md), the algorithms in [`effects.md`](effects.md), the
diagram sources in `docs/diagrams/*.mmd`, and a panel drawing per effect in
`docs/diagrams/fx/*.svg`. The Mermaid diagrams are embedded inline as well, from
the same strings, so nothing needs a build step to read.

`cargo xtask codegen` renders the parameter table itself into
`deepmind-midi/src/param/generated.rs`: the 242 parameters as an enum whose
discriminant is the NRPN number, their groups and ranges, the value tables with
the firmware each belongs to, and the controller map. It renders the program
layer into `deepmind-midi/src/program/generated.rs`: the value tables as Rust
types, and the 225 parameters that are not the program's name as a getter and a
setter each. The library cannot read TOML on a target with no filesystem and no
allocator, so the specification is compiled in rather than parsed.

Only data is generated. The types it fills and everything that acts on them are
hand-written beside it, so a specification change arrives in review as a changed
table rather than as changed logic, and a behaviour change cannot hide in a
regenerated file.

The same files generate the program layer: one Rust type per value table and one
pair of accessors per parameter, in
`deepmind-midi/src/program/generated.rs`. A correction gets made once.

### Keeping generated output honest

The failure mode is silent, so it is caught twice:

- `generated_documentation_is_current` compares the checked-in document against a
  fresh render, so a stale checkout fails `cargo test`.
- `generated_code_is_current` does the same for the library's parameter tables,
  so a spec file edited without regenerating fails the build rather than
  shipping a library that disagrees with its own specification.
- `cargo xtask docs --check` and `cargo xtask codegen --check` do the same
  without writing, and name the command to run. CI runs both for the clearer
  error.

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

## Decoding borrows, it does not copy

A bank program names dump is 2354 bytes on the wire. Handing a host an owned
copy of that would put an allocator on the path of every message, which is the
one thing a `no_std` library cannot do, so nothing on the way in is copied.

The decoder owns one buffer and reassembles frames into it. `Event::SysEx`
borrows that buffer and lives until the next byte is fed in, which is long
enough to parse and not long enough to hold. `Frame::parse` then borrows again:
a `Message` carries its bulk payload as a `&[u8]`, still packed. Unpacking is a
separate call, into a buffer the caller owns, because only the caller knows
where 245 bytes can go.

The buffer size is a const parameter defaulting to the longest documented frame.
A host that only sends NRPN edits and reads the edit buffer can say
`Decoder<300>` and save two kilobytes of stack; one that talks to something else
on the same port can say more. A frame that does not fit is dropped with one
error rather than truncated into something that would parse.

The decoder reports what arrived rather than tidying it. A note-on with velocity
zero stays a note-on, since the wire distinguishes them and some devices mean
the difference. Running status, real-time bytes inside a `SysEx` frame and a
status byte that ends one early are all handled, because they are what a MIDI
port actually delivers and a library that only worked on clean input would put
that work in every host.

## A file is a run of frames

A `.syx` file has no header, no index and no trailer. A preset pack is 128
program dumps end to end and a single patch is one dump, so reading a file is
reading frames and `syx` is the layer that walks them.

It walks the bytes directly rather than feeding them through `Decoder`. The
decoder exists to reassemble frames that arrive a few bytes at a time over a
port; a file is already whole, and walking it in place means a program dump is
borrowed from the file rather than copied into a buffer first. A payload is
seven-bit data, so an `F0` and the next `F7` delimit a frame exactly.

Files in the wild are padded, concatenated and appended to, so bytes outside a
frame are walked past. What is inside one is another matter: a frame that does
not parse is handed back as the error it failed with and the walk goes on, since
one bad frame is not a reason to lose the 127 good ones. That is why the
iterators yield a `Result` per item rather than refusing the file.

Writing does not promise to reproduce a file byte for byte, and says so. The
manual prints 278 packed bytes for a 242-byte program where padding gives 280
and truncating gives 277; this library pads, for the reason the packed codec
gives. A file written the other way reads back to identical programs and rewrites
to a different length. The programs are what a pack is; the padding is not, and
the golden test checks the programs always and the bytes where the two agree.

## Firmware is a dimension, not a footnote

Firmware 1.1 renumbered three value tables rather than only appending to them.
A program stored on 1.0 using modulation source 23 means something else read
back on 1.1. That is not a note for a human, it is a lookup key:

```rust
spec.table_for("mod_source", "1.0")    // 23 entries
spec.table_for("mod_source", "1.1")    // 25 entries
```

This is not a footnote-sized difference. 17 of the 23 modulation sources and 120
of the 130 modulation destinations changed meaning between the two versions, so
reading a 1.0 program with 1.1 tables mislabels almost everything. Only the FX
type list is nearly a pure extension.

A table declares the versions it covers: `"1.0"` for exactly that one, `"1.1+"`
for that one and later, and nothing at all for a table that has never changed.
The newest version is the default, so a caller who never mentions firmware gets
current hardware.

What the mechanism cannot do: a dump carries the comms protocol version, not the
firmware version, so nothing in a `.syx` file says which firmware wrote it. The
older tables are only reachable when the host knows the version another way.
Loading fails unless every table identifier resolves to exactly one table for
every listed version. A gap makes a table unreachable; an overlap makes the
answer depend on file order, which is how this sort of thing goes wrong
quietly.

Comms protocol version is a separate axis and stays separate: it decides how
many bytes a dump carries, not what a value means.

## Presentation is separate from protocol

Three files, because three different questions have three different sources.

`spec/effects.toml` says what a parameter is: its name as the manual writes it,
its range, its unit, whether the engine acts on modulation reaching it.

`spec/panels.toml` says what kind of control it is - continuous, switch or
selector - what it is called in full, and which slots belong together, such as
the two sides of the compressor or the two channels of the pitch shifter. All of
that is derived from `effects.toml` by this project, and the file says so.

`spec/layout.toml` says where the control goes, what it is, and what colour it
is. All of that is measured from the manual's figures, and the file says how.

The grid came from filling and labelling the ink in the 35 FX-page screenshots
of section 9.3 and reading off the circle centres: six columns at a 20-pixel
pitch, two rows, filled in slot order, a short row stopping rather than
spreading. The control and the colours came from the effect's own editor panel,
printed beside each screenshot: 29 are knobs, five are vertical faders, one is a
set of numeric displays, and four colours per panel are sampled by where they
sit - the case, the surface, the part a finger moves, and the one saturated
colour a label or an LED uses. Nothing there is a house style or a guess.

The two figures disagree about one thing, and the disagreement is the point. The
FX page draws every slot as a circle because a 128x64 display has room for one
shape. The panel draws what the effect actually is. A host that wants to look
like the synthesizer reads `grid.shape`; one that wants to look like the effect
reads `control` on the layout. Both are in the file, each labelled with where it
came from.

Splitting derived from measured is not bookkeeping. A derived field can be
argued with; a measured one can only be re-measured. Keeping them in separate
files means a host can tell which is which without reading prose.

The measurement earns its keep beyond drawing. It counts controls, and
`Spec::load` checks that count against `effects.toml`. That check found a
twelfth slot on `MoodFilter` that the text extraction had swallowed into the
eleventh row's description: the screenshot draws twelve circles and the table
had eleven. One algorithm out of 35 was wrong, and now the other 34 are
confirmed by something that had no part in building them.

What the manual still does not publish is geometry beyond that grid - no sizes
for anything but the circles, no fonts, no arrangement other than the one the
synthesizer itself uses. So `layout.toml` carries none, and a host is free to lay
the slots out its own way knowing what the hardware does.

## Routing is a graph, not a name

Ten fixed wirings of the four FX engines, chosen by one parameter. The manual
names them, and the names alone are not enough to act on: `Level` is defined as
the output level of effects "configured in parallel, or any effects which are the
last effect before reaching the output stage", so a host cannot label that control
without knowing where the current routing puts the slot.

`spec/routing.toml` carries all ten as edge lists, transcribed from the diagrams
in section 7.2.2. Loading checks each graph rather than trusting the
transcription: every slot has to be reachable from the FX block's input and have
a path to its output, and a topology's declared feedback flag has to match whether
its graph actually contains a loop. Ten small printed diagrams read by eye is
exactly the input that wants checking by machine.

## Strings are reachable by key

Every user-visible string in `spec/` is addressed by a stable path: a parameter
by its offset, an effect slot by its type and slot number, a value table entry
by its table and value. Nothing addresses a string by the string.

That is all translation needs from the data model, and it is already true. A
translation would be an overlay file keyed the same way, merged over the English
at load. Until someone actually wants one, English lives in the spec files and
there is no second mechanism to keep in sync. The point is that adding one later
is not a refactor.

What would not survive translation is anything that parses a display string.
Ranges are stored as their two ends and a unit, not as `"0.1 to 6.0 s"`, for
that reason among others.

## Raw values stay raw until measured

A parameter is one byte on the wire, and the manual gives the displayed value at
each end of its range but almost never the curve between. That curve cannot be
inferred: 201 of the 329 effect ranges start at or cross zero, which rules out a
logarithmic fit, and the manual documents a fader that starts at zero and is
still explicitly non-linear.

Almost never, rather than never, because the manual's figures give up more than
its prose does. Its PROG screenshots print a parameter's raw MIDI value next to
the value the synthesizer displays for it, which makes each one a measurement.
`spec/measurements.toml` collects the 29 of them the manual contains, and
[the specification](midi-spec.md#scaling-raw-values-to-displayed-values) works
through what they say. Most faders are linear. The three frequency parameters are
exponential, and not marginally: VCF Frequency reads 500.0 Hz where an
exponential sweep predicts 500.0005 and a straight line predicts 7717. Two faders
match neither, and the manual warns about neither.

None of that is a conversion, and none of it touches the 329 effect ranges, for
which the manual prints no screenshot and no graph. A single interior reading
fixes a curve only if the family is already known, and the two faders that match
nothing are what assuming the family costs.

So `Frequency::hz()` will exist only for parameters whose curve has been measured,
and `raw()` always works. A conversion that has not been measured is absent rather
than approximated: a plausible wrong number in front of a musician is worse than
an honest raw one, and it would be believed. A partially measured curve is absent
too, since a conversion that is right above the breakpoint and wrong below it is
the same trap wearing a better disguise.

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

Inbound traffic makes the same distinction, because a synthesizer whose knob has
been turned says so. An NRPN carries the parameter's whole value, so applying one
leaves the tracked program confirmed. A control change carries seven bits of a
value that usually has eight, so applying one leaves it assumed. The difference
is not pedantry: a host that redraws a fader from an assumed value and one that
writes it back to the synthesizer are doing different things with the same
number.

## The device tracks the sound, and reports the rest

`Device` holds two things: the edit buffer and what a device inquiry answered.
Everything else a frame can carry is reported by command and not decoded, which
is a boundary drawn by what is knowable and what fits.

Nothing is knowable about the globals, the patterns or the chord memories - the
manual gives those dumps a length and no offsets, as [Not yet
possible](#not-yet-possible-the-globals-and-the-sequencer) says. A bank of
program names is knowable and is two kilobytes, which is more than an event
queue with no allocator should carry for the hosts that never ask for one.

A stored program is neither: it decodes, and it is not the sound the synthesizer
is making, so it arrives as an event and leaves the tracked program alone. That
is also why a program-bearing event carries the program by value rather than
pointing at the tracked copy. A bank transfer overwrites that copy 128 times, and
an event meaning "look at the current one" would be worth nothing by the time
anyone looked.

Real-time bytes are dropped rather than queued. A running MIDI clock is
twenty-four messages a beat, none of which says anything about what the
synthesizer holds, and queuing them would starve the queue of the events that do.
A host that wants everything on the port drives `Decoder` itself, which is the
layer for it.

### Queuing is not sending

`request_edit_buffer` and its neighbours put an item in the outbound queue and
return. The bytes exist when `drain_tx` hands them to the host, and that is when
the request becomes outstanding and its timeout starts. A host that never drains
never times out, which is the truthful answer for a request that never went
anywhere.

The queue holds items rather than bytes, so a request costs a handful of bytes
and a whole-program edit costs 242 of them rather than the 2,904 bytes they
encode to. Both queues and the decoder's frame buffer are const parameters with
defaults, for the reason the decoder's already is: the right numbers differ by
two orders of magnitude between a desktop host and a microcontroller. An event
that does not fit is dropped and counted, and the count arrives as `Event::Lost`
once the queue has room again. Nothing is dropped quietly.

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
path from `DEEPMIND_PACKS` and skip when it is unset; the repository holds a
small synthetic fixture and its checksum. The fixture is what this library
writes, frozen: a change in the encoder arrives in review as a changed fixture
rather than as a file the next release cannot read. `UPDATE_FIXTURES=1` rewrites
it and prints the checksum to paste in, so agreeing to the change is an edit
somebody makes.

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
| 2 | `spec/effects.toml`: per-effect parameter names, presentation, FX routing |
| 3 | `wire` and `sysex`: framing, packed MS-bit codec, typed messages |
| 4 | `param`: code generated from `spec/`, typed values |
| 5 | `program`: the `Program` struct, its value types and typed accessors |
| 6 | `syx`: `.syx` files and preset packs |
| 7 | `device`: the state machine, events, timeouts, provenance, `edit` |
| 8 | `transport`: the blocking adapter |
| 9 | `deepmind-cli` |

Steps 1 to 7 have landed.

### Next: the transport

`transport`: the blocking adapter, which is a trait and a driver loop over the
state machine that is now there. It is the one part of this library that knows
what IO is, and it stays behind a feature flag for that reason - an async host
writes its own loop against the core in about thirty lines and keeps control of
cancellation and backpressure, which is the arrangement [Sans-IO](#sans-io)
argues for.

After it, `deepmind-cli`: a midir host that dumps banks, imports packs and
monitors traffic. It exists to prove the library is pleasant to use against real
hardware, which is a thing only real hardware can say.

### Not yet possible: the globals and the sequencer

`Global`, `Pattern` and the chord memories stay opaque payloads for now, and not
for want of a layer to put them in. The manual lists the 25 device-wide settings
but never says where in the 45-byte dump each one sits, and it gives the pattern
and chord dumps a length and nothing else. Naming those bytes needs hardware, not
another reading of the manual; `spec/globals.toml` records what is known and
marks the offsets as unknown rather than guessing at them.

The one dump besides the program that is documented well enough to decode is the
bank program names, which is 128 names of sixteen bytes and is where `BankNames`
comes from.
