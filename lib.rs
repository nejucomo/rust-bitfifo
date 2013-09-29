/* Copyright 2013 - Nathan Wilcox - Distributed under the terms of the
 * TGPPLv1 or at your option any later version.  See ./COPYING.TGPPL.rst
 * for details.
 */

/* A fifo... of bits! */

#[link(name = "bitfifo", vers = "0.1", author = "nejucomo@gmail.com")];
#[crate_type = "lib"];
extern mod std;
extern mod extra;


pub use bitfifo::BitFifo;
pub use item::{Pushable, Poppable, poppable_capacity};


// These basic items/macros are used in multiple files:
macro_rules! assert_le (
    ($smaller:expr , $bigger:expr) => (
        {
            let smaller_val = $smaller;
            let bigger_val = $bigger;
            if (smaller_val > bigger_val) {
                fail!("assertion failed: (%? <= %?)", smaller_val, bigger_val);
            }
        }
    )
)

pub type BitCount = uint;


// Public:
mod bitfifo;
mod item;

// Private:
mod bitbucket;

