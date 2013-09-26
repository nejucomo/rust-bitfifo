use std::uint;
use extra::ringbuf::RingBuf;
use extra::container::Deque;

// Local sub-modules:
use BitCount;
use item::Item;
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
    pub fn push<T: Item>(&mut self, source: &T, count: Option<BitCount>) {
        source.push_into(self, count);
    }

    pub fn pop<T: Item>(&mut self, count: Option<BitCount>) -> T {
        Item::pop_from(self, count)
    }

    // Concrete BitBucket push/pop:
    pub fn push_bitbucket(&mut self, source: &BitBucket) {
        let total = self.incoming.count + source.count;
        assert_le!(total, 2 * uint::bits);

        if total > uint::bits {
            let mut incoming = source.clone();

            let mut overflow = BitBucket::new();
            overflow.shift_in(&self.incoming);
            overflow.shift_in(&incoming.shift_out((uint::bits).checked_sub(&self.incoming.count).unwrap()));
            assert_eq!(overflow.count, uint::bits);
            self.queue.push_back(overflow.bits);

            self.incoming = incoming;

        } else {
            self.incoming.shift_in(source);
        }
    }

    pub fn pop_bitbucket(&mut self, count: BitCount) -> BitBucket {
        assert_le!(count, uint::bits);
        assert_le!(count, self.count());

        if count > self.outgoing.count {
            let mut result = self.outgoing.clone();

            match self.queue.pop_front() {
              None => {
                self.outgoing = self.incoming.clone();
                self.incoming = BitBucket::new();
              }
              Some(bits) => {
                self.outgoing = BitBucket { bits: bits, count: uint::bits }
              }
            }

            assert_le!(count, self.outgoing.count + result.count);
            assert_le!(result.count, count);
            let outcount = count.checked_sub(&result.count).unwrap();
            result.shift_in(&self.outgoing.shift_out(outcount));

            result

        } else {
            self.outgoing.shift_out(count)
        }
    }
}
