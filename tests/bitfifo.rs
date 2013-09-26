use self::utils::*;


// BitBucket push/pop tests:
#[test] fn fill_drain_bb_nibbles () { fill_drain_bb (bb_nibbles ()) }
#[test] fn fill_drain_bb_bytes   () { fill_drain_bb (bb_bytes   ()) }
#[test] fn fill_drain_bb_words   () { fill_drain_bb (bb_words   ()) }
#[test] fn lockstep_bb_nibbles   () { lockstep_bb   (bb_nibbles ()) }
#[test] fn lockstep_bb_bytes     () { lockstep_bb   (bb_bytes   ()) }
#[test] fn lockstep_bb_words     () { lockstep_bb   (bb_words   ()) }

// item push/pop tests:
#[test] fn fill_drain_nibble_items () { fill_drain_items (bb_nibbles ()) }
#[test] fn fill_drain_byte_items   () { fill_drain_items (bb_bytes   ()) }
#[test] fn fill_drain_word_items   () { fill_drain_items (bb_words   ()) }
#[test] fn lockstep_nibble_items   () { lockstep_items   (bb_nibbles ()) }
#[test] fn lockstep_byte_items     () { lockstep_items   (bb_bytes   ()) }
#[test] fn lockstep_word_items     () { lockstep_items   (bb_words   ()) }
#[test] fn push_pop_bb_word_vec    () { push_pop_vec     (bb_words   ()) }

#[test] fn fill_drain_uint_items () { fill_drain_items (uints ()) }
#[test] fn fill_drain_u64_items  () { fill_drain_items (u64s  ()) }
#[test] fn fill_drain_u32_items  () { fill_drain_items (u32s  ()) }
#[test] fn fill_drain_u16_items  () { fill_drain_items (u16s  ()) }
#[test] fn fill_drain_u8_items   () { fill_drain_items (u8s   ()) }
#[test] fn fill_drain_bool_items () { fill_drain_items (bools ()) }

#[test] fn lockstep_uint_items () { lockstep_items (uints ()) }
#[test] fn lockstep_u64_items  () { lockstep_items (u64s  ()) }
#[test] fn lockstep_u32_items  () { lockstep_items (u32s  ()) }
#[test] fn lockstep_u16_items  () { lockstep_items (u16s  ()) }
#[test] fn lockstep_u8_items   () { lockstep_items (u8s   ()) }
#[test] fn lockstep_bool_items () { lockstep_items (bools ()) }

#[test] fn push_pop_uint_vec () { push_pop_vec (uints ()) }
#[test] fn push_pop_u64_vec  () { push_pop_vec (u64s  ()) }
#[test] fn push_pop_u32_vec  () { push_pop_vec (u32s  ()) }
#[test] fn push_pop_u16_vec  () { push_pop_vec (u16s  ()) }
#[test] fn push_pop_u8_vec   () { push_pop_vec (u8s   ()) }
#[test] fn push_pop_bool_vec () { push_pop_vec (bools ()) }


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
