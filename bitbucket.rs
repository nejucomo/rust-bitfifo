use std::uint;


#[deriving(Clone)]
#[deriving(Eq)]
pub struct BitBucket {
    bits: uint,
    count: uint
}

impl BitBucket {
    pub fn empty() -> &'static BitBucket {
        static x: BitBucket = BitBucket { bits: 0, count: 0 };
        &x
    }

    pub fn new() -> BitBucket {
        BitBucket { bits: 0, count: 0 }
    }

    pub fn shift_in(&mut self, source: &BitBucket) {
        let total = self.count + source.count;
        assert_le!(total, uint::bits);
        self.bits = (self.bits << source.count) | source.bits;
        self.count = total;
    }

    pub fn shift_out(&mut self, count: uint) -> BitBucket {
        if (count == 0u) {
            return BitBucket::new();
        }

        assert_le!(count, self.count);

        let keep = self.count.checked_sub(&count).unwrap();
        let result = BitBucket {
            bits: self.bits >> keep,
            count: count
        };

        self.bits = self.bits & (1u << keep).checked_sub(&1u).unwrap();
        self.count = keep;

        result
    }
}
