use self::utils::*;

#[test]
fn shift_out_0() {
    use std::uint;
    use bitbucket::BitBucket;

    let src = BitBucket { bits: 0x0123456789abcdef, count: uint::bits };
    let mut bb = src.clone();
    assert_eq!(BitBucket::new(), bb.shift_out(0));
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
        dest.shift_in(full);
        assert_eq!(*dest, full);
    }

    pub fn shift_in_chunked(dest: &mut BitBucket) {
        for c in chunks.iter() {
            dest.shift_in(*c);
        }
        assert_eq!(*dest, full);
    }

    pub fn shift_out_all(src: &mut BitBucket) {
        let out = src.shift_out(full.count);
        assert_eq!(out, full);
        assert_eq!(*src, BitBucket::new());
    }

    pub fn shift_out_chunked(src: &mut BitBucket) {
        for c in chunks.iter() {
            let out = src.shift_out(c.count);
            assert_eq!(*c, out);
        }
    }
}

