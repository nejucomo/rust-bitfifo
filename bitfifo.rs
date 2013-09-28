use std::uint;
use extra::ringbuf::RingBuf;
use extra::container::Deque;

// Local sub-modules:
use BitCount;
use item::{Pushable, Poppable};
use bitbucket::BitBucket;



struct BitFifo {
    queue: RingBuf<uint>,
    incoming: BitBucket,
    outgoing: BitBucket
}

impl BitFifo {
    pub fn new() -> BitFifo {
        BitFifo {
            queue: RingBuf::new(),
            incoming: BitBucket::new(),
            outgoing: BitBucket::new()
        }
    }

    pub fn count(&self) -> BitCount {
        self.incoming.count + self.outgoing.count + uint::bits * self.queue.len()
    }

    // Polymorphic push/pop:
    pub fn push<T: Pushable>(&mut self, source: T) {
        self.push_opt_limit(source, None);
    }

    pub fn pop<T: Poppable>(&mut self) -> (T, BitCount) {
        self.pop_opt_limit(None)
    }

    pub fn push_limit<T: Pushable>(&mut self, source: T, limit: BitCount) {
        self.push_opt_limit(source, Some(limit));
    }

    pub fn pop_limit<T: Poppable>(&mut self, limit: BitCount) -> (T, BitCount) {
        self.pop_opt_limit(Some(limit))
    }

    fn push_opt_limit<T: Pushable>(&mut self, source: T, limit: Option<BitCount>) {
        source.push_into(self, limit);
    }

    fn pop_opt_limit<T: Poppable>(&mut self, limit: Option<BitCount>) -> (T, BitCount) {
        Poppable::pop_from(self, limit)
    }

    // Concrete BitBucket push/pop:
    pub fn push_bitbucket(&mut self, source: BitBucket) {
        let total = self.incoming.count + source.count;
        assert_le!(total, 2 * uint::bits);

        let (a, b) = self.incoming.merge_left(source, uint::bits);

        if a.count == uint::bits {
            self.queue.push_back(a.bits);
            self.incoming = b;
        } else {
            self.incoming = a;
        }
    }

    pub fn pop_bitbucket(&mut self, count: BitCount) -> BitBucket {
        assert_le!(count, uint::bits);
        assert_le!(count, self.count());

        if count <= self.outgoing.count {
            self.outgoing.pop_bits(count)
        } else {
            let tmp = self.pop_internal_bitbucket();
            let (a, b) = self.outgoing.merge_left(tmp, count);
            self.outgoing = b;
            a
        }
    }

    fn pop_internal_bitbucket(&mut self) -> BitBucket {
        match self.queue.pop_front() {
            None => {
                let result = self.incoming.clone();
                self.incoming = BitBucket::new();
                result
            }

            Some(bits) => {
                BitBucket { bits: bits, count: uint::bits }
            }
        }
    }
}
