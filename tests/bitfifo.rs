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

#[test] fn fill_drain_uint_items () { fill_drain_items (uints ()) }
#[test] fn lockstep_uint_items   () { lockstep_items   (uints ()) }


mod utils {
    use std::uint;
    use BitFifo;
    use bitfifoitem::BitFifoItem;
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

    pub fn uints() -> ~[uint] {
        let mut v = ~[];
        let mut word = 0x123456789abcdef0; // BUG: assumes 64 bit uint.

        for i in range(0u, 2^16) {
            v.push(i);
            word = (word << 1) + i;
        }
        v
    }

    // Test implementations, given a dataset:
    pub fn fill_drain_bb(bs: &[BitBucket]) {
        test_fill_drain(bs, push_bb, pop_bb)
    }

    pub fn lockstep_bb(bs: &[BitBucket]) {
        test_lockstep(bs, push_bb, pop_bb)
    }

    pub fn fill_drain_items<T: BitFifoItem>(xs: &[T]) {
        test_fill_drain(xs, push_item, pop_item)
    }

    pub fn lockstep_items<T: BitFifoItem>(xs: &[T]) {
        test_lockstep(xs, push_item, pop_item)
    }

    // Private:
    fn push_bb(fifo: &mut BitFifo, b: &BitBucket) -> uint {
        fifo.push_bitbucket(b);
        b.count
    }

    fn pop_bb(fifo: &mut BitFifo, b: &BitBucket) -> (BitBucket, uint) {
        let out = fifo.pop_bitbucket(b.count);
        (out, b.count)
    }

    fn push_item<T: BitFifoItem>(fifo: &mut BitFifo, x: &T) -> uint {
        fifo.push(x);
        x.bit_capacity()
    }

    fn pop_item<T: BitFifoItem>(fifo: &mut BitFifo, x: &T) -> (T, uint) {
        let count = x.bit_capacity();
        let out = fifo.pop_bits(count);
        (out, count)
    }

    fn test_fill_drain<T: Eq>(xs: &[T],
                              push: &fn(&mut BitFifo, &T) -> uint,
                              pop: &fn(&mut BitFifo, &T) -> (T, uint))
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
                            push: &fn(&mut BitFifo, &T) -> uint,
                            pop: &fn(&mut BitFifo, &T) -> (T, uint))
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