use ditherpunker_macros::simd_targets;

/// Macro that provides preset lane counts for common types based on CPU features.
///
/// Must be used inside a `#[multiversion]` function.
///
/// Usage:
///
/// ```
/// const LANES: usize = suggested_lanes!(f32);
/// ```
#[macro_export]
macro_rules! simd_width {
    (f32) => {
        multiversion::target::match_target!(
            "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl" => 16,
            "x86_64+avx2+fma" => 8,
            "x86_64+sse4.2" => 4,
            "x86_64+sse2" => 4,
            "aarch64+neon+sve" => 4,  // SVE can be wider but 4 is safe default
            "aarch64+neon" => 4,
            _ => 1,
        )
    };
    (f64) => {
        multiversion::target::match_target!(
            "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl" => 8,
            "x86_64+avx2+fma" => 4,
            "x86_64+sse4.2" => 2,
            "x86_64+sse2" => 2,
            "aarch64+neon+sve" => 2,
            "aarch64+neon" => 2,
            _ => 1,
        )
    };
    (u8) => {
        multiversion::target::match_target!(
            "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl" => 64,
            "x86_64+avx2+fma" => 32,
            "x86_64+sse4.2" => 16,
            "x86_64+sse2" => 16,
            "aarch64+neon+sve" => 16,
            "aarch64+neon" => 16,
            _ => 1,
        )
    };
    (u32) => {
        multiversion::target::match_target!(
            "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl" => 16,
            "x86_64+avx2+fma" => 8,
            "x86_64+sse4.2" => 4,
            "x86_64+sse2" => 4,
            "aarch64+neon+sve" => 4,
            "aarch64+neon" => 4,
            _ => 1,
        )
    };
}

/// Runtime version of simd_width which is not constrained to be inside
/// of a `#[multiversion]` function.
#[simd_targets]
pub fn suggested_simd_width<T: std::simd::SimdElement + 'static>() -> usize {
    use std::any::TypeId;

    let id: TypeId = std::any::TypeId::of::<T>();

    #[allow(clippy::if_same_then_else)]
    if id == TypeId::of::<f32>() {
        return simd_width!(f32);
    } else if id == TypeId::of::<f64>() {
        return simd_width!(f64);
    } else if id == TypeId::of::<u8>() {
        return simd_width!(u8);
    } else if id == TypeId::of::<u32>() {
        return simd_width!(u32);
    } else {
        unreachable!(
            "MISSING SIMD SUGGESTION: It's either missing from suggested_simd_width or simd_width macro, or there is a missmatch between simd_targets proc macro and suggested_simd_width"
        )
    }
}

#[cfg(test)]
pub mod tests {
    use super::suggested_simd_width;

    #[test]
    fn test_suggested_simd_width() {
        println!("f32: {}", suggested_simd_width::<f32>());
        println!("f64: {}", suggested_simd_width::<f64>());
        println!("u8: {}", suggested_simd_width::<u8>());
        println!("u32: {}", suggested_simd_width::<u32>());
    }
}
