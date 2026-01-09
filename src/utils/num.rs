/// Returns the closest power of 2 to an unsigned integer,
/// choosing between the previous and next powers of 2.
///
/// > preference to lower pow2
pub fn closest_pow_2<Integral>(n: Integral) -> Integral
where
    Integral: num_traits::int::PrimInt + num_traits::Unsigned,
{
    if n.is_zero() {
        return Integral::one();
    }

    let leading_zeros = n.leading_zeros() as usize;
    let bits = size_of::<Integral>() * 8;
    let highest_bit_pos = bits - 1 - leading_zeros;

    let lower = Integral::one() << highest_bit_pos;
    // overflow protection, if no leading 0 => no upper pow2
    if leading_zeros == 0 {
        return lower;
    }

    let upper = lower << 1;
    if (n - lower) <= (upper - n) {
        lower
    } else {
        upper
    }
}
