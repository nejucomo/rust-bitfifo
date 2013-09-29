/* Copyright 2013 - Nathan Wilcox - Distributed under the terms of the
 * TGPPLv1 or at your option any later version.  See ./COPYING.TGPPL.rst
 * for details.
 */

use std::uint;
use BitCount;


#[deriving(Clone)]
#[deriving(Eq)]
pub struct BitBucket {
    bits: uint,
    count: BitCount
}

impl BitBucket {
    pub fn new() -> BitBucket {
        BitBucket { bits: 0, count: 0 }
    }

    pub fn merge_left(&self, right: BitBucket, count: BitCount) -> (BitBucket, BitBucket) {
        assert_le!(count, uint::bits);
        assert_le!(self.count, count); // If this were false, use pop_bits instead.

        let total = self.count + right.count;

        if count >= total {
            // Shove all bits to the self:
            (BitBucket {
                    bits: (self.bits << right.count) | right.bits,
                    count: total
                },
             BitBucket::new())

        } else {
            let tomove = count - self.count;
            let keepright = right.count - tomove;

            (BitBucket {
                    bits: (self.bits << tomove) | right.bits >> keepright,
                    count: count
                },
             BitBucket {
                    bits: right.bits & ((1 << keepright) - 1),
                    count: keepright
                })
        }
    }

    pub fn pop_bits(&mut self, count: BitCount) -> BitBucket {
        if (count == 0u) {
            return BitBucket::new();
        }

        assert_le!(count, self.count);

        let keep = self.count.checked_sub(&count).unwrap();

        let result = BitBucket {
            bits: self.bits >> keep,
            count: count
        };

        self.bits = self.bits & ((1u << keep) - 1);
        self.count = keep;

        result
    }
}

#[cfg(test)]
mod tests {
    use self::utils::*;

    #[test]
    fn pop_bits_0() {
        use std::uint;
        use super::BitBucket;

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
        use super::super::BitBucket;

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
}
