//! The device state machine and the simulated unit, fed a cable.
//!
//! [`Device::feed`] and [`Synth::feed`] are the only ways into the NRPN
//! assembler and the inquiry reply parser, and everything a host knows about
//! the unit comes out of them. The input is a script rather than a byte string:
//! things to ask for, a stream to feed in given chunks, and a clock to move, so
//! requests, replies, timeouts and noise interleave in every order. The two
//! ends are wired to each other as well, so what the host asks reaches a unit
//! that answers, and the answer comes back through the decoder the noise goes
//! through.
//!
//! Beyond not panicking: the outbound queue's capacity is a constant and a
//! drain nothing refused empties it, nothing is outstanding that was never
//! sent, and both event queues drain in bounded time.

#![no_main]

use std::convert::Infallible;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use deepmind_midi::device::{DEFAULT_EVENT_DEPTH, Device, MAX_PENDING};
use deepmind_midi::ids::{Bank, DeviceId, ProgramNumber, ProtocolVersion, Slot};
use deepmind_midi::param::ParamId;
use deepmind_midi::program::{Program, ProgramName};
use deepmind_midi::sim::{DEFAULT_HEARD_DEPTH, Synth};
use deepmind_midi::syx::{File, MAX_BANK_LEN, Writer};

/// Programs the unit holds in bank A, so a request for a slot has an answer.
const STORED: u8 = 4;

/// One thing the host does between reads of the port.
#[derive(Arbitrary, Debug)]
enum Action {
    Identity,
    EditBuffer,
    Program {
        bank: u8,
        number: u8,
    },
    Bank {
        bank: u8,
        first: u8,
        last: u8,
    },
    /// Seeds a tracked program, so an edit has something to diff against.
    Assume {
        version_7: bool,
    },
    Edit {
        parameter: u8,
        value: u8,
    },
    Reset,
}

/// A script for the host.
#[derive(Arbitrary, Debug)]
struct Input {
    /// Which unit the host addresses. The simulated one is unit 0.
    unit: u8,
    /// What the host does, one per step.
    actions: Vec<Action>,
    /// Bytes as they would arrive from a port.
    stream: Vec<u8>,
    /// Sizes of the reads that carry them, cycled. Empty means one read.
    chunks: Vec<u8>,
    /// How far the clock moves at each step, cycled. Empty means it stands
    /// still.
    ticks: Vec<u16>,
}

/// Writes the pack the unit's memory is made of.
fn library() -> Vec<u8> {
    let mut bytes = vec![0; MAX_BANK_LEN];
    let mut writer = Writer::new(&mut bytes, DeviceId::Unit(0));
    for index in 0..STORED {
        let mut program = Program::new(ProtocolVersion::V7);
        program.set_name(ProgramName::new(&format!("Stored {index}")).expect("a legal name"));
        let number = ProgramNumber::new(index).expect("in range");
        writer
            .push_program(Bank::A, number, &program)
            .expect("the buffer fits a bank");
    }
    let written = writer.finish();
    bytes.truncate(written);
    bytes
}

fn bank(byte: u8) -> Bank {
    Bank::new(byte & 0x07).expect("masked into range")
}

fn number(byte: u8) -> ProgramNumber {
    ProgramNumber::new(byte & 0x7F).expect("masked into range")
}

/// Does what `action` says. Returns whether a request was queued.
fn act(device: &mut Device, action: Action) -> bool {
    match action {
        Action::Identity => device.request_identity().is_ok(),
        Action::EditBuffer => device.request_edit_buffer().is_ok(),
        Action::Program { bank: b, number: n } => device
            .request_program(Slot::new(bank(b), number(n)))
            .is_ok(),
        Action::Bank {
            bank: b,
            first,
            last,
        } => device
            .request_bank(bank(b), number(first), number(last))
            .is_ok(),
        Action::Assume { version_7 } => {
            let version = if version_7 {
                ProtocolVersion::V7
            } else {
                ProtocolVersion::V6
            };
            device.assume_program(Program::new(version));
            false
        }
        Action::Edit { parameter, value } => {
            let parameter = ParamId::ALL[usize::from(parameter) % ParamId::ALL.len()];
            let _ = device.edit(|program| {
                program.set_clamped(parameter, value);
            });
            false
        }
        Action::Reset => {
            device.reset();
            false
        }
    }
}

/// The invariants that hold at every step, whatever was fed.
fn check(device: &Device, synth: &Synth<File<'_>>, tx: usize, replies: usize, sent: usize) {
    assert_eq!(
        device.queued() + device.room(),
        tx,
        "the outbound capacity moved"
    );
    assert_eq!(
        synth.queued() + synth.room(),
        replies,
        "the reply capacity moved"
    );
    let outstanding = device.outstanding().count();
    assert!(
        outstanding <= sent,
        "{outstanding} outstanding after {sent} requests were queued"
    );
    assert!(outstanding <= MAX_PENDING);
}

fuzz_target!(|input: Input| {
    let id = match input.unit % 4 {
        0 | 1 => DeviceId::Unit(0),
        2 => DeviceId::Unit(1),
        _ => DeviceId::Broadcast,
    };
    let pack = library();
    let mut device: Device = Device::new(id);
    let mut synth: Synth<File<'_>> = Synth::with_library(
        DeviceId::Unit(0),
        Program::new(ProtocolVersion::V7),
        File::new(&pack),
    );

    let tx = device.queued() + device.room();
    let replies = synth.queued() + synth.room();
    let mut sent = 0;
    let mut now = 0_u64;

    let mut actions = input.actions.into_iter();
    let mut chunks = input
        .chunks
        .iter()
        .map(|size| usize::from(*size).max(1))
        .cycle();
    let mut ticks = input.ticks.iter().copied().cycle();
    let mut rest = input.stream.as_slice();

    loop {
        let action = actions.next();
        if action.is_none() && rest.is_empty() {
            break;
        }
        if let Some(action) = action {
            if act(&mut device, action) {
                sent += 1;
            }
        }

        if !rest.is_empty() {
            let take = chunks.next().unwrap_or(rest.len()).min(rest.len());
            let (head, tail) = rest.split_at(take);
            device.feed(head);
            synth.feed(head);
            rest = tail;
        }

        now = now.saturating_add(u64::from(ticks.next().unwrap_or(0)));
        device.tick(now);
        check(&device, &synth, tx, replies, sent);

        // The cable, both ways. Nothing on it refuses, so both queues empty.
        device
            .drain_tx(|bytes| {
                synth.feed(bytes);
                Ok::<(), Infallible>(())
            })
            .expect("the cable never refuses");
        assert_eq!(
            device.queued(),
            0,
            "a drain nothing refused left items queued"
        );
        synth
            .drain_tx(|bytes| {
                device.feed(bytes);
                Ok::<(), Infallible>(())
            })
            .expect("the cable never refuses");
        assert_eq!(
            synth.queued(),
            0,
            "a drain nothing refused left replies queued"
        );
        check(&device, &synth, tx, replies, sent);

        // Both pollers hand out what is queued and then one count of what
        // was lost, so neither can go on longer than that.
        let mut polled = 0;
        while device.poll_event().is_some() {
            polled += 1;
            assert!(
                polled <= DEFAULT_EVENT_DEPTH + 1,
                "poll_event is not draining"
            );
        }
        let mut polled = 0;
        while synth.poll_heard().is_some() {
            polled += 1;
            assert!(
                polled <= DEFAULT_HEARD_DEPTH + 1,
                "poll_heard is not draining"
            );
        }
    }
});
