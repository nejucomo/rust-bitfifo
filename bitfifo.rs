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

        let (a, b) = bbops::merge_left(self.incoming, source, uint::bits);

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
            let outgoing = &mut self.outgoing;
            bbops::pop_bits(outgoing, count)
        } else {
            let tmp = self.pop_internal_bitbucket();
            let (a, b) = bbops::merge_left(self.outgoing, tmp, count);
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


// Private bitbucket operations:
mod bbops {
    use std::uint;
    use BitCount;
    use bitbucket::BitBucket;

    pub fn merge_left(left: BitBucket, right: BitBucket, count: BitCount) -> (BitBucket, BitBucket) {
        assert_le!(count, uint::bits);
        assert_le!(left.count, count); // If this were false, use pop_bits instead.

        let total = left.count + right.count;

        if count >= total {
            // Shove all bits to the left:
            (BitBucket {
                    bits: (left.bits << right.count) | right.bits,
                    count: total
                },
             BitBucket::new())

        } else {
            let tomove = count - left.count;
            let keepright = right.count - tomove;

            (BitBucket {
                    bits: (left.bits << tomove) | right.bits >> keepright,
                    count: count
                },
             BitBucket {
                    bits: right.bits & ((1 << keepright) - 1),
                    count: keepright
                })
        }
    }

    pub fn pop_bits(source: &mut BitBucket, count: BitCount) -> BitBucket {
        if (count == 0u) {
            return BitBucket::new();
        }

        assert_le!(count, source.count);

        let keep = source.count.checked_sub(&count).unwrap();

        let result = BitBucket {
            bits: source.bits >> keep,
            count: count
        };

        source.bits = source.bits & ((1u << keep) - 1);
        source.count = keep;

        result
    }

    #[cfg(test)]
    mod tests {
        use self::utils::*;
        use super::pop_bits;

        #[test]
        fn pop_bits_0() {
            use std::uint;
            use bitbucket::BitBucket;

            let src = BitBucket { bits: 0x0123456789abcdef, count: uint::bits };
            let mut bb = src.clone();
            assert_eq!(BitBucket::new(), pop_bits(&mut bb, 0));
            assert_eq!(src, bb);
        }

        #[test] fn all_in_all_out() { iotest(shift_in_all, shift_out_all) }
        #[test] fn all_in_chunked_out() { iotest(shift_in_all, shift_out_chunked) }
        #[test] fn chunked_in_all_out() { iotest(shift_in_chunked, shift_out_all) }
        #[test] fn chunked_in_chunked_out() { iotest(shift_in_chunked, shift_out_chunked) }


        mod utils {
            use bitbucket::BitBucket;
            use super::super::{merge_left, pop_bits};

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
                shift_in(dest, full);
                assert_eq!(*dest, full);
            }

            pub fn shift_in_chunked(dest: &mut BitBucket) {
                for c in chunks.iter() {
                    shift_in(dest, *c);
                }
                assert_eq!(*dest, full);
            }

            pub fn shift_out_all(src: &mut BitBucket) {
                let out = pop_bits(src, full.count);
                assert_eq!(out, full);
                assert_eq!(*src, BitBucket::new());
            }

            pub fn shift_out_chunked(src: &mut BitBucket) {
                for c in chunks.iter() {
                    let out = pop_bits(src, c.count);
                    assert_eq!(*c, out);
                }
            }

            fn shift_in(dest: &mut BitBucket, source: BitBucket) {
                let (a, b) = merge_left(*dest, source, dest.count + source.count);
                assert_eq!(b, BitBucket::new());
                *dest = a;
            }
        }
    }
}
