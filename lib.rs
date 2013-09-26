/* A fifo... of bits! */

#[link(name = "bitfifo", vers = "0.1", author = "nejucomo@gmail.com")];
#[crate_type = "lib"];
extern mod std;
extern mod extra;


pub use bitfifo::BitFifo;
pub use item::Item;
pub use bitbucket::BitBucket;


// This is used in multiple files:
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


mod bitfifo;
mod item;
mod bitbucket;

#[cfg(test)] mod tests;
