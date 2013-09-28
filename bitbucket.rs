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

    pub fn merge_left(&self, other: BitBucket, count: BitCount) -> (BitBucket, BitBucket) {
        assert_le!(count, uint::bits);
        assert_le!(self.count, count); // If this were false, use pop_bits instead.

        let total = self.count + other.count;

        if count >= total {
            // Shove all bits to the left:
            (BitBucket {
                    bits: (self.bits << other.count) | other.bits,
                    count: total
                },
             BitBucket::new())

        } else {
            let tomove = count - self.count;
            let keepright = other.count - tomove;

            (BitBucket {
                    bits: (self.bits << tomove) | other.bits >> keepright,
                    count: count
                },
             BitBucket {
                    bits: other.bits & ((1 << keepright) - 1),
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
