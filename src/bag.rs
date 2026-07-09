//! 7-bag randomizer (TETR.IO / guideline).

use crate::piece::PieceKind;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug)]
pub struct SevenBag {
    bag: Vec<PieceKind>,
    queue: Vec<PieceKind>,
}

impl SevenBag {
    pub fn new() -> Self {
        let mut s = Self {
            bag: Vec::new(),
            queue: Vec::new(),
        };
        // Prefill so next queue of 5 is always ready
        while s.queue.len() < 14 {
            s.refill_bag();
            s.queue.extend(s.bag.drain(..));
        }
        s
    }

    fn refill_bag(&mut self) {
        self.bag = PieceKind::ALL.to_vec();
        self.bag.shuffle(&mut thread_rng());
    }

    pub fn next(&mut self) -> PieceKind {
        if self.queue.len() < 7 {
            self.refill_bag();
            self.queue.extend(self.bag.drain(..));
        }
        self.queue.remove(0)
    }

    /// Peek next `n` pieces without consuming.
    pub fn peek(&self, n: usize) -> Vec<PieceKind> {
        self.queue.iter().take(n).copied().collect()
    }
}

impl Default for SevenBag {
    fn default() -> Self {
        Self::new()
    }
}
