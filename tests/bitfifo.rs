// High-level tests:
mod highlevel {
    #[test]
    fn gather_bytes_into_u32() {
        use BitCount;
        use BitFifo;

        let mut fifo = BitFifo::new();

        fifo.push(&0xABu8, None);
        fifo.push(&0xCDu8, None);
        fifo.push(&0xEFu8, None);
        fifo.push(&0x89u8, None);
        assert_eq!(fifo.count(), 32);

        let (x, count): (u32, BitCount) = fifo.pop(None);

        assert_eq!(fifo.count(), 0);
        assert_eq!(count, 32);
        assert_eq!(x, 0xABCDEF89u32);
    }

    #[test]
    fn split_u32_into_bytes() {
        use BitCount;
        use BitFifo;

        let mut fifo = BitFifo::new();

        fifo.push(&0xABCDEF89u32, None);
        assert_eq!(fifo.count(), 32);

        let (a, acnt): (u8, BitCount) = fifo.pop(None);
        assert_eq!(fifo.count(), 24);
        assert_eq!(acnt, 8);
        assert_eq!(a, 0xABu8);

        let (b, bcnt): (u8, BitCount) = fifo.pop(None);
        assert_eq!(fifo.count(), 16);
        assert_eq!(bcnt, 8);
        assert_eq!(b, 0xCDu8);

        let (c, ccnt): (u8, BitCount) = fifo.pop(None);
        assert_eq!(fifo.count(), 8);
        assert_eq!(ccnt, 8);
        assert_eq!(c, 0xEFu8);

        let (d, dcnt): (u8, BitCount) = fifo.pop(None);
        assert_eq!(fifo.count(), 0);
        assert_eq!(dcnt, 8);
        assert_eq!(d, 0x89u8);
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
                #[test] fn push_pop() { push_pop_vec($datagen()) }
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
    use item::Item;
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

    pub fn fill_drain_items<T: Item>(xs: &[T]) {
        test_fill_drain(xs, push_item, pop_item)
    }

    pub fn lockstep_items<T: Item>(xs: &[T]) {
        test_lockstep(xs, push_item, pop_item)
    }

    pub fn push_pop_vec<T: Item>(xs: ~[T]) {
        let xcount = xs.bit_count();
        let mut fifo = BitFifo::new();
        fifo.push(&xs, None);
        assert_eq!(fifo.count(), xcount);
        let (ys, count) = fifo.pop(None);
        assert_eq!(xs, ys);
        assert_eq!(xcount, count);
    }

    // Private:
    fn push_bb(fifo: &mut BitFifo, b: &BitBucket) -> BitCount {
        fifo.push_bitbucket(b);
        b.count
    }

    fn pop_bb(fifo: &mut BitFifo, b: &BitBucket) -> (BitBucket, BitCount) {
        let out = fifo.pop_bitbucket(b.count);
        (out, b.count)
    }

    fn push_item<T: Item>(fifo: &mut BitFifo, x: &T) -> BitCount {
        fifo.push(x, None);
        x.bit_count()
    }

    fn pop_item<T: Item>(fifo: &mut BitFifo, x: &T) -> (T, BitCount) {
        let incount = x.bit_count();
        let (out, outcount) = fifo.pop(Some(incount));
        assert_eq!(incount, outcount);
        (out, outcount)
    }

    fn test_fill_drain<T: Eq>(xs: &[T],
                              push: &fn(&mut BitFifo, &T) -> BitCount,
                              pop: &fn(&mut BitFifo, &T) -> (T, BitCount))
    {
        let mut fifo = BitFifo::new();
        let mut count = 0;

        // Fill:
        for x in xs.iter() {
            count += push(&mut fifo, x);
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

    fn test_lockstep<T: Eq>(xs: &[T],
                            push: &fn(&mut BitFifo, &T) -> BitCount,
                            pop: &fn(&mut BitFifo, &T) -> (T, BitCount))
    {
        let mut fifo = BitFifo::new();

        // Fill/drain in lockstep:
        for x in xs.iter() {
            assert_eq!(fifo.count(), 0);
            let c = push(&mut fifo, x);
            assert_eq!(fifo.count(), c);
            let (out, _) = pop(&mut fifo, x);
            assert_eq!(&out, x);
            assert_eq!(fifo.count(), 0);
        }
    }
}
