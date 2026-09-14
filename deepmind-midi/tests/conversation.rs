//! A host and a synthesizer, talking.
//!
//! Every other test in this repository drives one layer with bytes somebody
//! wrote down. These drive [`Device`] and [`Transport`] against
//! [`sim::Synth`](deepmind_midi::sim::Synth), which answers the way
//! `spec/messages.toml` says a unit answers and is never told what the host is
//! about to ask.
//!
//! **What this can find, and what it cannot.** Both ends are built from the same
//! specification, so a round trip here says that this library agrees with itself
//! and nothing about a `DeepMind`. What it does reach is the seam between the
//! layers: a request whose answer resolves the wrong outstanding item, a dump
//! that lands in the wrong place, a run that stops one short, a timeout that
//! fires when the answer did arrive. None of those is visible from one side.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed expectation is the test failure"
)]

use std::collections::VecDeque;
use std::convert::Infallible;

use deepmind_midi::device::{Device, Event, Request};
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::sim::{Empty, Heard, Library, Synth};
use deepmind_midi::syx::{self, File, Writer};
use deepmind_midi::transport::{Clock, Error as TransportError, Port, Transport};

/// A clock that only moves when something waits on it.
struct Fake(u64);

impl Clock for Fake {
    fn now_ms(&mut self) -> u64 {
        self.0
    }

    fn sleep_ms(&mut self, milliseconds: u64) {
        self.0 = self.0.saturating_add(milliseconds);
    }
}

/// A MIDI cable with a synthesizer on the far end.
///
/// The unit answers as soon as it is spoken to, so what the host reads next is
/// whatever the last thing it sent was worth. A cable with no latency is not a
/// realistic one, and [`Deaf`] is the other half of that picture.
struct Cable<L> {
    synth: Synth<L>,
    inbound: VecDeque<u8>,
}

impl<L: Library> Cable<L> {
    fn new(synth: Synth<L>) -> Self {
        Self {
            synth,
            inbound: VecDeque::new(),
        }
    }
}

impl<L: Library> Port for Cable<L> {
    type Error = Infallible;

    fn send(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        let Self { synth, inbound } = self;
        synth.feed(bytes);
        synth
            .drain_tx(|reply| {
                inbound.extend(reply);
                Ok::<(), Infallible>(())
            })
            .expect("collecting never refuses");
        Ok(())
    }

    fn receive(&mut self, into: &mut [u8]) -> Result<usize, Infallible> {
        let taken = self.inbound.len().min(into.len());
        for slot in into.iter_mut().take(taken) {
            *slot = self.inbound.pop_front().unwrap_or(0);
        }
        Ok(taken)
    }
}

/// A cable to a unit that hears everything and answers nothing, which is what a
/// synthesizer that is powered off but still plugged in looks like.
struct Deaf {
    synth: Synth,
}

impl Port for Deaf {
    type Error = Infallible;

    fn send(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.synth.feed(bytes);
        Ok(())
    }

    fn receive(&mut self, _into: &mut [u8]) -> Result<usize, Infallible> {
        Ok(0)
    }
}

fn named(name: &str) -> Program {
    let mut program = Program::new(ProtocolVersion::V7).expect("a known version");
    program.set_name(ProgramName::new(name).expect("a legal name"));
    program
}

/// Writes a preset pack holding `count` programs in bank A.
fn pack(count: u8) -> Vec<u8> {
    let mut bytes = vec![0; syx::MAX_BANK_LEN];
    let mut writer = Writer::new(&mut bytes, DeviceId::Unit(0));
    for index in 0..count {
        let number = ProgramNumber::new(index).expect("in range");
        writer
            .push_program(Bank::A, number, &named(&format!("Preset {index}")))
            .expect("the buffer fits a bank");
    }
    let written = writer.finish();
    bytes.truncate(written);
    bytes
}

fn transport<L: Library>(synth: Synth<L>) -> Transport<Cable<L>, Fake> {
    Transport::new(Device::new(DeviceId::Unit(0)), Cable::new(synth), Fake(0))
}

#[test]
fn a_host_reads_the_sound_the_unit_is_making() {
    let synth: Synth = Synth::new(DeviceId::Unit(0), named("Bass Sweep"));
    let mut host = transport(synth);

    let program = host.edit_buffer().expect("the unit answers");

    assert_eq!(program.name().as_str(), "Bass Sweep");
    assert!(host.device().program().is_confirmed());
}

#[test]
fn a_host_reads_the_firmware_the_unit_reports() {
    let synth: Synth = Synth::new(DeviceId::Unit(0), named("Bass Sweep"));
    let expected = synth.identity();
    let mut host = transport(synth);

    assert_eq!(host.identity().expect("the unit answers"), expected);
}

/// The loop a librarian writes: read a bank off the unit, one dump at a time.
#[test]
fn a_host_reads_a_run_of_stored_programs() {
    let bytes = pack(8);
    let synth: Synth<File<'_>> =
        Synth::with_library(DeviceId::Unit(0), named("Edit Buffer"), File::new(&bytes));
    let mut host = transport(synth);

    let mut read = Vec::new();
    let count = host
        .bank(
            Bank::A,
            ProgramNumber::FIRST,
            ProgramNumber::new(7).expect("in range"),
            |slot, program| read.push((slot.number.get(), program.name().as_str().to_owned())),
        )
        .expect("the unit answers every slot");

    assert_eq!(count, 8);
    for (index, (number, name)) in read.iter().enumerate() {
        assert_eq!(usize::from(*number), index);
        assert_eq!(name, &format!("Preset {index}"));
    }
}

/// A preset pack is the unit's memory, so what the host reads back out is what
/// went in. A `.syx` file is the only place a program crosses between two
/// programs that do not share an address space, which makes this the one round
/// trip worth running end to end.
#[test]
fn a_pack_read_back_off_the_unit_is_the_pack_that_went_in() {
    let bytes = pack(4);
    let slot = Slot::new(Bank::A, ProgramNumber::new(2).expect("in range"));
    let synth: Synth<File<'_>> =
        Synth::with_library(DeviceId::Unit(0), named("Edit Buffer"), File::new(&bytes));
    let mut host = transport(synth);

    let read = host.program(slot).expect("the unit holds that slot");

    let original = File::new(&bytes)
        .programs()
        .filter_map(Result::ok)
        .find(|entry| entry.slot == Some(slot))
        .expect("the pack holds that slot");
    assert_eq!(read, original.program);
}

/// A stored program is not the sound the unit is making, and a host that
/// confuses the two shows a user the wrong patch.
#[test]
fn a_stored_program_does_not_become_the_tracked_one() {
    let bytes = pack(2);
    let synth: Synth<File<'_>> =
        Synth::with_library(DeviceId::Unit(0), named("Edit Buffer"), File::new(&bytes));
    let mut host = transport(synth);

    let held = host.edit_buffer().expect("the unit answers");
    let stored = host
        .program(Slot::new(Bank::A, ProgramNumber::FIRST))
        .expect("the unit holds that slot");

    assert_eq!(held.name().as_str(), "Edit Buffer");
    assert_eq!(stored.name().as_str(), "Preset 0");
    assert_eq!(
        host.device()
            .program()
            .value()
            .expect("a program is tracked")
            .name()
            .as_str(),
        "Edit Buffer"
    );
}

/// The full round: read what it holds, change one thing, and read it back to
/// find the change there.
#[test]
fn an_edit_reaches_the_unit_and_survives_a_resync() {
    let synth: Synth = Synth::new(DeviceId::Unit(0), named("Bass Sweep"));
    let mut host = transport(synth);

    host.edit_buffer().expect("the unit answers");
    let changed = host
        .edit(|program| program.set_lfo1_rate(64))
        .expect("a program is known and the value fits");
    assert_eq!(changed, 1);
    assert!(host.device().program().is_assumed());

    // Nothing confirms an edit but a dump, and the dump now carries it.
    let program = host.edit_buffer().expect("the unit answers");

    assert_eq!(program.get(ParamId::Lfo1Rate), 64);
    assert!(host.device().program().is_confirmed());
}

/// Turning a knob on the unit is traffic the host never asked for, and it has to
/// land on the tracked program anyway.
#[test]
fn what_the_unit_volunteers_reaches_the_host() {
    let synth: Synth = Synth::new(DeviceId::Unit(0), named("Bass Sweep"));
    let mut host = transport(synth);
    host.edit_buffer().expect("the unit answers");

    host.port_mut()
        .synth
        .turn(ParamId::Lfo1Rate, 100)
        .expect("a value it accepts");
    // The unit speaks when the cable next carries anything.
    host.port_mut().send(&[]).expect("the cable never refuses");
    host.pump().expect("the cable never refuses");

    assert!(matches!(
        host.poll_event(),
        Some(Event::Parameter {
            parameter: ParamId::Lfo1Rate,
            value: 100,
        })
    ));
    assert_eq!(
        host.device()
            .program()
            .value()
            .expect("a program is tracked")
            .get(ParamId::Lfo1Rate),
        100
    );
}

/// A unit that never answers is the case a host has to survive, and the only
/// thing that makes it happen is time passing.
#[test]
fn a_unit_that_never_answers_times_out() {
    let mut host: Transport<Deaf, Fake> = Transport::new(
        Device::new(DeviceId::Unit(0)),
        Deaf {
            synth: Synth::new(DeviceId::Unit(0), named("Bass Sweep")),
        },
        Fake(0),
    );

    let result = host.edit_buffer();

    assert!(matches!(
        result,
        Err(TransportError::Timeout(Request::EditBuffer))
    ));
    // It heard the question. It simply said nothing.
    assert!(matches!(
        host.port_mut().synth.poll_heard(),
        Some(Heard::Request { answered: true, .. })
    ));
}

/// A port delivers whatever the driver hands it, and a `SysEx` dump is longer
/// than most of them deliver at once.
#[test]
fn a_dump_split_across_reads_arrives_whole() {
    let mut synth: Synth = Synth::new(DeviceId::Unit(0), named("Bass Sweep"));
    let mut host: Device = Device::new(DeviceId::Unit(0));

    host.request_edit_buffer().expect("room to queue it");
    host.drain_tx(|bytes| {
        synth.feed(bytes);
        Ok::<(), Infallible>(())
    })
    .expect("collecting never refuses");

    // One byte at a time, which is the worst a port can do short of stopping.
    synth
        .drain_tx(|reply| {
            for byte in reply {
                host.feed(&[*byte]);
            }
            Ok::<(), Infallible>(())
        })
        .expect("collecting never refuses");

    let Some(Event::EditBuffer(program)) = host.poll_event() else {
        panic!("the dump arrived whole");
    };
    assert_eq!(program.name().as_str(), "Bass Sweep");
}

/// Two units on one port, which is what a device ID is for.
#[test]
fn a_unit_answers_only_what_is_addressed_to_it() {
    let mut first: Synth = Synth::new(DeviceId::Unit(0), named("First"));
    let mut second: Synth = Synth::new(DeviceId::Unit(1), named("Second"));
    let mut host: Device = Device::new(DeviceId::Unit(1));

    host.request_edit_buffer().expect("room to queue it");
    host.drain_tx(|bytes| {
        // One cable, both units listening.
        first.feed(bytes);
        second.feed(bytes);
        Ok::<(), Infallible>(())
    })
    .expect("collecting never refuses");

    assert_eq!(first.queued(), 0);
    assert_eq!(second.queued(), 1);

    second
        .drain_tx(|bytes| {
            host.feed(bytes);
            Ok::<(), Infallible>(())
        })
        .expect("collecting never refuses");

    let Some(Event::EditBuffer(program)) = host.poll_event() else {
        panic!("the addressed unit answered");
    };
    assert_eq!(program.name().as_str(), "Second");
}

/// A unit with nothing stored answers a program request with nothing, and the
/// host has to time out rather than wait forever.
#[test]
fn a_slot_the_unit_does_not_hold_times_out() {
    let synth: Synth<Empty> = Synth::new(DeviceId::Unit(0), named("Bass Sweep"));
    let mut host = transport(synth);

    let result = host.program(Slot::new(Bank::A, ProgramNumber::FIRST));

    assert!(matches!(result, Err(TransportError::Timeout(_))));
}
