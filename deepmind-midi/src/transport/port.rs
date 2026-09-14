//! The MIDI connection, as the driver loop needs to see it.

/// A MIDI connection the driver loop can read from and write to.
///
/// Two methods, because two is what driving the state machine takes. Everything
/// a real backend also has - port enumeration, virtual ports, connection state,
/// reconnection - stays on the host's side of this trait, where it belongs: a
/// library that opened ports would have to have an opinion about all of it.
///
/// # Implementing it
///
/// Most backends deliver inbound bytes on a callback thread rather than to a
/// reader, so the usual shape is a queue the callback fills and
/// [`receive`](Port::receive) drains:
///
/// ```
/// use std::sync::mpsc::{Receiver, TryRecvError};
///
/// use deepmind_midi::transport::Port;
///
/// struct Connection {
///     outbound: Vec<Vec<u8>>,      // whatever the backend sends with
///     inbound: Receiver<Vec<u8>>,  // filled by the backend's callback thread
///     partial: Vec<u8>,            // what did not fit in the last read
/// }
///
/// impl Port for Connection {
///     type Error = std::io::Error;
///
///     fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
///         self.outbound.push(bytes.to_vec());
///         Ok(())
///     }
///
///     fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
///         if self.partial.is_empty() {
///             match self.inbound.try_recv() {
///                 Ok(message) => self.partial = message,
///                 Err(TryRecvError::Empty) => return Ok(0),
///                 Err(TryRecvError::Disconnected) => return Ok(0),
///             }
///         }
///         let taken = self.partial.len().min(into.len());
///         into[..taken].copy_from_slice(&self.partial[..taken]);
///         self.partial.drain(..taken);
///         Ok(taken)
///     }
/// }
/// ```
///
/// The buffer a [`Transport`](super::Transport) offers is as long as the
/// decoder's frame buffer, so the longest frame the library can read fits in one
/// call and the `partial` above never holds anything in practice. It is still
/// worth writing: the contract is "as much as fits", not "all or nothing".
pub trait Port {
    /// What the connection fails with.
    type Error;

    /// Sends one message.
    ///
    /// Called once per message, which is the granularity a MIDI port wants. A
    /// backend that would rather make fewer, larger writes buffers here.
    ///
    /// # Errors
    ///
    /// Returns whatever the connection failed with. The item stays queued, so a
    /// host that recovers can drain again without having lost it.
    fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// Reads whatever has arrived, up to `into.len()` bytes, without blocking.
    ///
    /// Returning `Ok(0)` means nothing has arrived yet. It never means the
    /// connection has ended: a MIDI port that is open and quiet is the normal
    /// case, and a port that has gone away reports that as an error.
    ///
    /// Bytes may be split across calls however the backend likes. The decoder
    /// behind this reassembles frames, so a chunking that cuts one in half costs
    /// nothing.
    ///
    /// # Errors
    ///
    /// Returns whatever the connection failed with.
    fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error>;
}

impl<P: Port + ?Sized> Port for &mut P {
    type Error = P::Error;

    fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        (**self).send(bytes)
    }

    fn receive(&mut self, into: &mut [u8]) -> Result<usize, Self::Error> {
        (**self).receive(into)
    }
}
