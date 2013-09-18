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
            overflow.shift_in(&incoming.shift_out(safe_sub(uint::bits, self.incoming.count)));
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
            assert!(count >= result.count);
            result.shift_in(&self.outgoing.shift_out(safe_sub(count, result.count)));

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
        if (count == 0u) {
            return BitBucket::new();
        }

        assert!(count <= self.count);

        let keep = safe_sub(self.count, count);
        let result = BitBucket {
            bits: self.bits >> keep,
            count: count
        };

        self.bits = self.bits & safe_sub(1 << keep, 1);
        self.count = keep;

        result
    }
}

fn safe_sub(a: uint, b: uint) -> uint {
    assert!(a >= b);
    a - b
}

#[cfg(test)]
mod tests {
    mod BitFifo {
        use self::utils::*;

        #[test] fn fill_drain_nibbles() { fill_drain(nibbles()) }
        #[test] fn lockstep_nibbles() { lockstep(nibbles()) }
        #[test] fn fill_drain_bytes() { fill_drain(bytes()) }
        #[test] fn lockstep_bytes() { lockstep(bytes()) }
        #[test] fn fill_drain_words() { fill_drain(words()) }
        #[test] fn lockstep_words() { lockstep(words()) }

        // Test only the first or first two elements of each data set to hunt for edge cases:
        #[test] fn fill_drain_nibbles_1() { on_first_n(fill_drain, nibbles(), 1) }
        #[test] fn lockstep_nibbles_1() { on_first_n(lockstep, nibbles(), 1) }
        #[test] fn fill_drain_bytes_1() { on_first_n(fill_drain, bytes(), 1) }
        #[test] fn lockstep_bytes_1() { on_first_n(lockstep, bytes(), 1) }
        #[test] fn fill_drain_words_1() { on_first_n(fill_drain, words(), 1) }
        #[test] fn lockstep_words_1() { on_first_n(lockstep, words(), 1) }

        #[test] fn fill_drain_nibbles_2() { on_first_n(fill_drain, nibbles(), 2) }
        #[test] fn lockstep_nibbles_2() { on_first_n(lockstep, nibbles(), 2) }
        #[test] fn fill_drain_bytes_2() { on_first_n(fill_drain, bytes(), 2) }
        #[test] fn lockstep_bytes_2() { on_first_n(lockstep, bytes(), 2) }
        #[test] fn fill_drain_words_2() { on_first_n(fill_drain, words(), 2) }
        #[test] fn lockstep_words_2() { on_first_n(lockstep, words(), 2) }


        mod utils {
            use std::uint;
            use BitFifo;
            use BitBucket;

            // datasets:
            pub fn nibbles() -> ~[BitBucket] {
                let mut v = ~[];
                for nib in range(0u, 2^4) {
                    v.push(BitBucket { bits: nib, count: 4 });
                }
                v
            }

            pub fn bytes() -> ~[BitBucket] {
                let mut v = ~[];
                for byte in range(0u, 2^8) {
                    v.push(BitBucket { bits: byte, count: 8 });
                }
                v
            }

            pub fn words() -> ~[BitBucket] {
                let mut v = ~[];
                let mut word = 0x0123456789abcdef; // BUG: assumes 64 bit uint.

                for i in range(0u, 2^16) {
                    v.push(BitBucket { bits: word, count: uint::bits });
                    word = (word << 1) + i;
                }
                v
            }

            // Test implementations, given a dataset:
            // A HOF for searching for edge case bugs:
            pub fn on_first_n(f: &fn (&[BitBucket]), bs: &[BitBucket], n: uint) {
                f(bs.slice(0, n))
            }

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
                    let out = fifo.pop(b.count);
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
        fn shift_out_0() {
            use std::uint;
            use BitBucket;

            let src = BitBucket { bits: 0x0123456789abcdef, count: uint::bits };
            let mut bb = src.clone();
            assert_eq!(*BitBucket::empty(), bb.shift_out(0));
            assert_eq!(src, bb);
        }

        #[test] fn all_in_all_out() { iotest(shift_in_all, shift_out_all) }
        #[test] fn all_in_chunked_out() { iotest(shift_in_all, shift_out_chunked) }
        #[test] fn chunked_in_all_out() { iotest(shift_in_chunked, shift_out_all) }
        #[test] fn chunked_in_chunked_out() { iotest(shift_in_chunked, shift_out_chunked) }


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

