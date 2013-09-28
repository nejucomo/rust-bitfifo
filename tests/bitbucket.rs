use self::utils::*;

#[test]
fn pop_bits_0() {
    use std::uint;
    use bitbucket::BitBucket;

    let src = BitBucket { bits: 0x0123456789abcdef, count: uint::bits };
    let mut bb = src.clone();
    assert_eq!(BitBucket::new(), bb.pop_bits(0));
    assert_eq!(src, bb);
}

#[test] fn all_in_all_out() { iotest(shift_in_all, shift_out_all) }
#[test] fn all_in_chunked_out() { iotest(shift_in_all, shift_out_chunked) }
#[test] fn chunked_in_all_out() { iotest(shift_in_chunked, shift_out_all) }
#[test] fn chunked_in_chunked_out() { iotest(shift_in_chunked, shift_out_chunked) }


mod utils {
    use bitbucket::BitBucket;

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
        let out = src.pop_bits(full.count);
        assert_eq!(out, full);
        assert_eq!(*src, BitBucket::new());
    }

    pub fn shift_out_chunked(src: &mut BitBucket) {
        for c in chunks.iter() {
            let out = src.pop_bits(c.count);
            assert_eq!(*c, out);
        }
    }

    fn shift_in(dest: &mut BitBucket, source: BitBucket) {
        let (a, b) = dest.merge_left(source, dest.count + source.count);
        assert_eq!(b, BitBucket::new());
        *dest = a;
    }
}

