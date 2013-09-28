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
}
