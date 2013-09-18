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
            assert_eq!(overflow.count, uint::bits);
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

            assert!(count <= self.outgoing.count + result.count);
            result.shift_in(&self.outgoing.shift_out(count - result.count));

            result

        } else {
            self.outgoing.shift_out(count)
        }
    }
}


#[deriving(Clone)]
#[deriving(Eq)]
pub struct BitBucket {
    bits: uint,
    count: uint
}

impl BitBucket {
    fn empty() -> &'static BitBucket {
        static x: BitBucket = BitBucket { bits: 0, count: 0 };
        &x
    }

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

#[cfg(test)]
mod tests {
    mod BitFifo {
        use self::utils::*;

        #[test]
        fn fill_drain_nibbles() { fill_drain(nibbles()) }

        #[test]
        fn lockstep_nibbles() { lockstep(nibbles()) }

        mod utils {
            use BitFifo;
            use BitBucket;

            // datasets:
            pub fn nibbles() -> ~[BitBucket] {
                let mut v = ~[];
                for nib in range(0u, 16) {
                    v.push(BitBucket { bits: nib, count: 4 });
                }
                v
            }

            // Test implementations, given a dataset:
            pub fn fill_drain(bs: &[BitBucket]) {
                let mut fifo = BitFifo::new();
                let mut count = 0;

                // Fill:
                for b in bs.iter() {
                    fifo.push(b);
                    count += b.count;
                    assert_eq!(fifo.count(), count);
                }

                // Drain:
                for b in bs.iter() {
                    let out = fifo.pop(4);
                    assert_eq!(out, *b);
                    count -= out.count;
                    assert_eq!(fifo.count(), count);
                }
                assert_eq!(fifo.count(), 0);
            }

            pub fn lockstep(bs: &[BitBucket]) {
                let mut fifo = BitFifo::new();

                // Fill/drain in lockstep:
                for b in bs.iter() {
                    assert_eq!(fifo.count(), 0);
                    fifo.push(b);
                    assert_eq!(fifo.count(), b.count);
                    let out = fifo.pop(b.count);
                    assert_eq!(out, *b);
                    assert_eq!(fifo.count(), 0);
                }
            }
        }
    }

    mod BitBucket {
        use self::utils::*;

        #[test]
        fn all_in_all_out() { iotest(shift_in_all, shift_out_all) }

        #[test]
        fn all_in_chunked_out() { iotest(shift_in_all, shift_out_chunked) }

        #[test]
        fn chunked_in_all_out() { iotest(shift_in_chunked, shift_out_all) }

        #[test]
        fn chunked_in_chunked_out() { iotest(shift_in_chunked, shift_out_chunked) }


        mod utils {
            use BitBucket;

            static full: BitBucket = BitBucket { bits: 0x1b, count: 5 };
            static chunks: [BitBucket, .. 3] = [
                BitBucket { bits: 0x3, count: 2 },
                BitBucket { bits: 0x1, count: 2 },
                BitBucket { bits: 0x1, count: 1 },
                ];

            pub fn iotest(inop: &fn(&mut BitBucket), outop: &fn(&mut BitBucket)) {
                let bb = &mut BitBucket::new();
                inop(bb);
                outop(bb);
            }

            pub fn shift_in_all(dest: &mut BitBucket) {
                dest.shift_in(&full);
                assert_eq!(*dest, full);
            }

            pub fn shift_in_chunked(dest: &mut BitBucket) {
                for c in chunks.iter() {
                    dest.shift_in(c);
                }
                assert_eq!(*dest, full);
            }

            pub fn shift_out_all(src: &mut BitBucket) {
                let out = src.shift_out(full.count);
                assert_eq!(out, full);
                assert_eq!(*src, *BitBucket::empty());
            }

            pub fn shift_out_chunked(src: &mut BitBucket) {
                for c in chunks.iter() {
                    let out = src.shift_out(c.count);
                    assert_eq!(*c, out);
                }
            }
        }
    }
}

