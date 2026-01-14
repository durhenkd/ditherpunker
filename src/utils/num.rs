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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closest_pow_2_zero() {
        assert_eq!(closest_pow_2(0u32), 1);
        assert_eq!(closest_pow_2(0u64), 1);
        assert_eq!(closest_pow_2(0usize), 1);
    }

    #[test]
    fn test_closest_pow_2_exact_powers() {
        assert_eq!(closest_pow_2(1u32), 1);
        assert_eq!(closest_pow_2(2u32), 2);
        assert_eq!(closest_pow_2(4u32), 4);
        assert_eq!(closest_pow_2(8u32), 8);
        assert_eq!(closest_pow_2(16u32), 16);
        assert_eq!(closest_pow_2(32u32), 32);
        assert_eq!(closest_pow_2(64u32), 64);
        assert_eq!(closest_pow_2(128u32), 128);
        assert_eq!(closest_pow_2(256u32), 256);
        assert_eq!(closest_pow_2(1024u32), 1024);
    }

    #[test]
    fn test_closest_pow_2_prefer_lower() {
        // When distance is equal, prefer lower power of 2
        assert_eq!(closest_pow_2(3u32), 2); // 3-2=1, 4-3=1, prefer lower
        assert_eq!(closest_pow_2(6u32), 4); // 6-4=2, 8-6=2, prefer lower
        assert_eq!(closest_pow_2(12u32), 8); // 12-8=4, 16-12=4, prefer lower
        assert_eq!(closest_pow_2(24u32), 16); // 24-16=8, 32-24=8, prefer lower
    }

    #[test]
    fn test_closest_pow_2_closer_to_lower() {
        assert_eq!(closest_pow_2(5u32), 4); // 5-4=1, 8-5=3, choose 4
        assert_eq!(closest_pow_2(9u32), 8); // 9-8=1, 16-9=7, choose 8
        assert_eq!(closest_pow_2(17u32), 16); // 17-16=1, 32-17=15, choose 16
        assert_eq!(closest_pow_2(33u32), 32); // 33-32=1, 64-33=31, choose 32
    }

    #[test]
    fn test_closest_pow_2_closer_to_upper() {
        assert_eq!(closest_pow_2(7u32), 8); // 7-4=3, 8-7=1, choose 8
        assert_eq!(closest_pow_2(15u32), 16); // 15-8=7, 16-15=1, choose 16
        assert_eq!(closest_pow_2(31u32), 32); // 31-16=15, 32-31=1, choose 32
        assert_eq!(closest_pow_2(63u32), 64); // 63-32=31, 64-63=1, choose 64
    }

    #[test]
    fn test_closest_pow_2_max_value() {
        // Test with maximum value (no upper power of 2 available)
        assert_eq!(closest_pow_2(u32::MAX), 1u32 << 31);
        assert_eq!(closest_pow_2(u64::MAX), 1u64 << 63);

        // Test values near maximum
        assert_eq!(closest_pow_2(u32::MAX - 1), 1u32 << 31);
        assert_eq!(closest_pow_2((1u32 << 31) + 1), 1u32 << 31);
    }

    #[test]
    fn test_closest_pow_2_different_integer_types() {
        assert_eq!(closest_pow_2(10u8), 8u8);
        assert_eq!(closest_pow_2(10u16), 8u16);
        assert_eq!(closest_pow_2(10u32), 8u32);
        assert_eq!(closest_pow_2(10u64), 8u64);
        assert_eq!(closest_pow_2(10usize), 8usize);
    }

    #[test]
    fn test_closest_pow_2_large_values() {
        assert_eq!(closest_pow_2(1000u32), 1024); // 1000-512=488, 1024-1000=24, choose 1024
        assert_eq!(closest_pow_2(2000u32), 2048); // 2000-1024=976, 2048-2000=48, choose 2048
        assert_eq!(closest_pow_2(100000u32), 131072); // 100000-65536=34464, 131072-100000=31072, choose 131072
        assert_eq!(closest_pow_2(1000000u32), 1048576); // 1000000-524288=475712, 1048576-1000000=48576, choose 1048576
    }
}
