// Check for uint subtraction overflow with an assertion:

pub fn safe_sub(a: uint, b: uint) -> uint {
    assert_le!(b, a);
    a - b
}

