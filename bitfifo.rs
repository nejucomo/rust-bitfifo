/* A fifo... of bits! */

#[link(name = "bitfifo", vers = "0.1", author = "nejucomo@gmail.com")];
#[crate_type = "lib"];
extern mod std;
extern mod extra;

use std::uint;
use extra::ringbuf::RingBuf;
use extra::container::Deque;


struct BitFifo {
    queue: RingBuf<uint>,
    incoming: BitBucket,
    outgoing: BitBucket
}

impl BitFifo {
    fn new() -> BitFifo {
        BitFifo {
            queue: RingBuf::new(),
            incoming: BitBucket::new(),
            outgoing: BitBucket::new()
        }
    }

    fn count(&self) -> uint {
        self.incoming.count + self.outgoing.count + uint::bits * self.queue.len()
    }

    fn push(&mut self, source: &BitBucket) {
        let total = self.incoming.count + source.count;
        assert!(total <= 2 * uint::bits);

        if total > uint::bits {
            let mut incoming = source.clone();

            let mut overflow = BitBucket::new();
            overflow.shift_in(&self.incoming);
            overflow.shift_in(&incoming.shift_out(uint::bits - self.incoming.count));
            assert!(overflow.count == uint::bits);
            self.queue.push_back(overflow.bits);

            self.incoming = incoming;

        } else {
            self.incoming.shift_in(source);
        }
    }

    fn pop(&mut self, count: uint) -> BitBucket {
        assert!(count <= uint::bits);
        assert!(count <= self.count());

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

            assert!(count < self.outgoing.count + result.count);
            result.shift_in(&self.outgoing.shift_out(count - result.count));

            result

        } else {
            self.outgoing.shift_out(count)
        }
    }
}


#[deriving(Clone)]
struct BitBucket {
    bits: uint,
    count: uint
}

impl BitBucket {
    fn new() -> BitBucket {
        BitBucket { bits: 0, count: 0 }
    }

    fn shift_in(&mut self, source: &BitBucket) {
        let total = self.count + source.count;
        assert!(total <= uint::bits);
        self.bits = (self.bits << source.count) | source.bits;
        self.count = total;
    }

    fn shift_out(&mut self, count: uint) -> BitBucket {
        assert!(count <= self.count);

        let keep = self.count - count;
        let result = BitBucket {
            bits: self.bits >> keep,
            count: count
        };

        self.bits = self.bits & ((1 << keep) - 1);
        self.count = keep;

        result
    }
}
