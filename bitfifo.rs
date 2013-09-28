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


#[cfg(test)]
pub mod tests {
    // High-level tests:
    pub mod highlevel {
        #[test]
        fn gather_bytes_into_u32() {
            use BitCount;
            use BitFifo;

            let mut fifo = BitFifo::new();

            fifo.push(0xABu8);
            fifo.push(0xCDu8);
            fifo.push(0xEFu8);
            fifo.push(0x89u8);
            assert_eq!(fifo.count(), 32);

            let (x, count): (u32, BitCount) = fifo.pop();

            assert_eq!(fifo.count(), 0);
            assert_eq!(count, 32);
            assert_eq!(x, 0xABCDEF89u32);
        }

        #[test]
        fn split_u32_into_bytes() {
            use BitCount;
            use BitFifo;

            let mut fifo = BitFifo::new();

            fifo.push(0xABCDEF89u32);
            assert_eq!(fifo.count(), 32);

            let (a, acnt): (u8, BitCount) = fifo.pop();
            assert_eq!(fifo.count(), 24);
            assert_eq!(acnt, 8);
            assert_eq!(a, 0xABu8);

            let (b, bcnt): (u8, BitCount) = fifo.pop();
            assert_eq!(fifo.count(), 16);
            assert_eq!(bcnt, 8);
            assert_eq!(b, 0xCDu8);

            let (c, ccnt): (u8, BitCount) = fifo.pop();
            assert_eq!(fifo.count(), 8);
            assert_eq!(ccnt, 8);
            assert_eq!(c, 0xEFu8);

            let (d, dcnt): (u8, BitCount) = fifo.pop();
            assert_eq!(fifo.count(), 0);
            assert_eq!(dcnt, 8);
            assert_eq!(d, 0x89u8);
        }

        #[test]
        fn the_answer_in_bools() {
            use BitCount;
            use BitFifo;

            let mut fifo = BitFifo::new();

            do 3.times {
                fifo.push(true);
                fifo.push(false);
            }

            let (answer, count): (uint, BitCount) = fifo.pop();

            assert_eq!(answer, 42u);
            assert_eq!(count, 6u);
        }
    }


    // BitBucket push/pop tests:
    mod bitbucket {
        macro_rules! bitbucket_dataset_tests(
            ($modname:ident +{ $datagen:expr }) => (
                mod $modname {
                    use super::super::utils::*;

                    #[test] fn fill_drain () { fill_drain_bb ($datagen ()) }
                    #[test] fn lockstep   () { lockstep_bb   ($datagen ()) }
                }
            )
        )

        bitbucket_dataset_tests!(words   +{ bb_words   })
        bitbucket_dataset_tests!(bytes   +{ bb_bytes   })
        bitbucket_dataset_tests!(nibbles +{ bb_nibbles })
    }

    // item data-driven tests:
    mod item {
        macro_rules! item_dataset_tests(
            ($modname:ident +{ $datagen:expr }) => (
                mod $modname {
                    use super::super::utils::*;

                    #[test] fn fill_drain() { fill_drain_items($datagen()) }
                    #[test] fn lockstep() { lockstep_items($datagen()) }
                    #[test] fn push_pop() { push_pop_unique_vec($datagen()) }
                }
            );

            ($modname:ident -{ $datagen:expr }) => (
                mod $modname {
                    use super::super::utils::*;

                    #[test] fn fill_drain() { fill_drain_items($datagen()) }
                    #[test] fn lockstep() { lockstep_items($datagen()) }
                }
            )
        )

        item_dataset_tests!(bb_words   +{ bb_words   })
        item_dataset_tests!(bb_bytes   -{ bb_bytes   })
        item_dataset_tests!(bb_nibbles -{ bb_nibbles })
        item_dataset_tests!(uint       +{ uints      })
        item_dataset_tests!(u64        +{ u64s       })
        item_dataset_tests!(u32        +{ u32s       })
        item_dataset_tests!(u16        +{ u16s       })
        item_dataset_tests!(u8         +{ u8s        })
        item_dataset_tests!(bool       +{ bools      })
    }


    mod utils {
        use std::uint;
        use BitCount;
        use BitFifo;
        use item::{Pushable, Poppable};
        use bitbucket::BitBucket;

        // datasets:
        pub fn bb_nibbles() -> ~[BitBucket] {
            let mut v = ~[];
            for nib in range(0u, 2^4) {
                v.push(BitBucket { bits: nib, count: 4 });
            }
            v
        }

        pub fn bb_bytes() -> ~[BitBucket] {
            let mut v = ~[];
            for byte in range(0u, 2^8) {
                v.push(BitBucket { bits: byte, count: 8 });
            }
            v
        }

        pub fn bb_words() -> ~[BitBucket] {
            let mut v = ~[];
            let mut word = 0x0123456789abcdef; // BUG: assumes 64 bit uint.

            for i in range(0u, 2^16) {
                v.push(BitBucket { bits: word, count: uint::bits });
                word = (word << 1) + i;
            }
            v
        }

        pub fn bools() -> ~[bool] { ~[false, true] }


        macro_rules! uint_data_generator(
            ($genname:ident, $T:ty) => (
                pub fn $genname() -> ~[$T] {
                    let mut v = ~[];
                    let mut word = 0x123456789abcdef0;

                    for i in range(0u, 2^16) {
                        v.push(word as $T);
                        word = (word << 1) + i;
                    }
                    v
                }
            )
        )

        uint_data_generator!(uints, uint)
        uint_data_generator!(u64s, u64)
        uint_data_generator!(u32s, u32)
        uint_data_generator!(u16s, u16)
        uint_data_generator!(u8s, u8)

        // Test implementations, given a dataset:
        pub fn fill_drain_bb(bs: &[BitBucket]) {
            test_fill_drain(bs, push_bb, pop_bb)
        }

        pub fn lockstep_bb(bs: &[BitBucket]) {
            test_lockstep(bs, push_bb, pop_bb)
        }

        pub fn fill_drain_items<T: Eq + Clone + Pushable + Poppable>(xs: &[T]) {
            test_fill_drain(xs, push_item, pop_item)
        }

        pub fn lockstep_items<T: Eq + Clone + Pushable + Poppable>(xs: &[T]) {
            test_lockstep(xs, push_item, pop_item)
        }

        pub fn push_pop_unique_vec<T: Eq + Pushable + Poppable>(xs: ~[T]) {
            let xcount = xs.bit_count();
            let xsborrow: &[T] = xs;
            let mut fifo = BitFifo::new();
            fifo.push(xsborrow);
            assert_eq!(fifo.count(), xcount);
            let (ys, count) = fifo.pop();
            assert_eq!(&xs, &ys);
            assert_eq!(xcount, count);
        }

        // Private:
        fn push_bb(fifo: &mut BitFifo, b: BitBucket) -> BitCount {
            fifo.push_bitbucket(b);
            b.count
        }

        fn pop_bb(fifo: &mut BitFifo, b: &BitBucket) -> (BitBucket, BitCount) {
            let out = fifo.pop_bitbucket(b.count);
            (out, b.count)
        }

        fn push_item<T: Pushable>(fifo: &mut BitFifo, x: T) -> BitCount {
            let result = x.bit_count();
            fifo.push(x);
            result
        }

        fn pop_item<T: Pushable + Poppable>(fifo: &mut BitFifo, x: &T) -> (T, BitCount) {
            let incount = x.bit_count();
            let (out, outcount) = fifo.pop_limit(incount);
            assert_eq!(incount, outcount);
            (out, outcount)
        }

        fn test_fill_drain<T: Eq + Clone>(xs: &[T],
                                          push: &fn(&mut BitFifo, T) -> BitCount,
                                          pop: &fn(&mut BitFifo, &T) -> (T, BitCount))
        {
            let mut fifo = BitFifo::new();
            let mut count = 0;

            // Fill:
            for x in xs.iter() {
                count += push(&mut fifo, x.clone());
                assert_eq!(fifo.count(), count);
            }

            // Drain:
            for x in xs.iter() {
                let (out, c) = pop(&mut fifo, x);
                assert_eq!(&out, x);
                count -= c;
                assert_eq!(fifo.count(), count);
            }
            assert_eq!(fifo.count(), 0);
        }

        fn test_lockstep<T: Eq + Clone>(xs: &[T],
                                        push: &fn(&mut BitFifo, T) -> BitCount,
                                        pop: &fn(&mut BitFifo, &T) -> (T, BitCount))
        {
            let mut fifo = BitFifo::new();

            // Fill/drain in lockstep:
            for x in xs.iter() {
                assert_eq!(fifo.count(), 0);
                let c = push(&mut fifo, x.clone());
                assert_eq!(fifo.count(), c);
                let (out, _) = pop(&mut fifo, x);
                assert_eq!(&out, x);
                assert_eq!(fifo.count(), 0);
            }
        }
    }
}
