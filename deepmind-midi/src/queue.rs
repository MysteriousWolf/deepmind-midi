//! A fixed-capacity ring, which is what a queue is without an allocator.

/// A first-in, first-out queue of `N` items, held inline.
///
/// Nothing here grows. The capacity is the host's to choose and the overflow is
/// the host's to see: [`push`](Queue::push) reports whether the item was taken,
/// and the caller decides what saying no means.
#[derive(Debug)]
pub(crate) struct Queue<T, const N: usize> {
    slots: [Option<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> Queue<T, N> {
    /// Builds an empty queue.
    pub(crate) const fn new() -> Self {
        Self {
            slots: [const { None }; N],
            head: 0,
            len: 0,
        }
    }

    /// Returns how many items are waiting.
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    /// Returns how many more items fit.
    pub(crate) const fn remaining(&self) -> usize {
        N - self.len
    }

    /// Adds an item, returning whether there was room for it.
    pub(crate) fn push(&mut self, item: T) -> bool {
        if self.len >= N {
            return false;
        }
        let index = (self.head + self.len) % N;
        match self.slots.get_mut(index) {
            Some(slot) => {
                *slot = Some(item);
                self.len += 1;
                true
            }
            None => false,
        }
    }

    /// Takes the oldest item.
    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let item = self.slots.get_mut(self.head).and_then(Option::take);
        self.head = (self.head + 1) % N;
        self.len -= 1;
        item
    }

    /// Copies the oldest item without taking it.
    pub(crate) fn peek(&self) -> Option<T>
    where
        T: Copy,
    {
        if self.len == 0 {
            return None;
        }
        self.slots.get(self.head).copied().flatten()
    }

    /// Drops everything waiting.
    pub(crate) fn clear(&mut self) {
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::Queue;

    #[test]
    fn items_come_out_in_the_order_they_went_in() {
        let mut queue: Queue<u8, 4> = Queue::new();
        assert_eq!(queue.len(), 0);
        for value in 1..=3 {
            assert!(queue.push(value));
        }
        assert_eq!(queue.len(), 3);
        assert_eq!(queue.peek(), Some(1));
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn a_full_queue_says_so_rather_than_dropping_the_oldest() {
        let mut queue: Queue<u8, 2> = Queue::new();
        assert!(queue.push(1));
        assert!(queue.push(2));
        assert!(!queue.push(3));
        assert_eq!(queue.remaining(), 0);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.remaining(), 1);
    }

    #[test]
    fn the_ring_wraps() {
        let mut queue: Queue<u8, 3> = Queue::new();
        for round in 0..10 {
            assert!(queue.push(round));
            assert_eq!(queue.pop(), Some(round));
        }
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn a_queue_of_nothing_holds_nothing() {
        let mut queue: Queue<u8, 0> = Queue::new();
        assert!(!queue.push(1));
        assert_eq!(queue.pop(), None);
        assert_eq!(queue.peek(), None);
    }

    #[test]
    fn clearing_empties_it() {
        let mut queue: Queue<u8, 4> = Queue::new();
        assert!(queue.push(1));
        assert!(queue.push(2));
        queue.clear();
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop(), None);
    }
}
