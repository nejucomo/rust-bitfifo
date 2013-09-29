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

    // Concrete uint push/pop:
    pub fn push_uint(&mut self, bits: uint, count: BitCount) {
        self.push_bitbucket(BitBucket { bits: bits, count: count });
    }

    pub fn pop_uint(&mut self, count: BitCount) -> (uint, BitCount) {
        let bb = self.pop_bitbucket(count);
        (bb.bits, bb.count)
    }

    // Private push/pop implementation:
    fn push_bitbucket(&mut self, source: BitBucket) {
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

    fn pop_bitbucket(&mut self, count: BitCount) -> BitBucket {
        assert_le!(count, uint::bits);
        assert_le!(count, self.count());

        if count <= self.outgoing.count {
            let outgoing = &mut self.outgoing;
            outgoing.pop_bits(count)
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


    // item data-driven tests:
    mod item {
        macro_rules! item_dataset_tests(
            ($modname:ident, $datagen:expr) => (
                mod $modname {
                    use super::super::utils::*;

                    #[test] fn fill_drain() { fill_drain_items($datagen()) }
                    #[test] fn lockstep() { lockstep_items($datagen()) }
                    #[test] fn push_pop() { push_pop_unique_vec($datagen()) }
                }
            )
        )

        item_dataset_tests!(uint, uints)
        item_dataset_tests!(u64 , u64s )
        item_dataset_tests!(u32 , u32s )
        item_dataset_tests!(u16 , u16s )
        item_dataset_tests!(u8  , u8s  )
        item_dataset_tests!(bool, bools)
    }


    mod utils {
        use BitCount;
        use BitFifo;
        use item::{Pushable, Poppable};
        use bitbucket::BitBucket;

        // datasets:
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
