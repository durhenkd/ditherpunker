/// Tests and Benchmarks to figure out what combination of features
/// actually enables simd optimizations without intrinsics.
///
/// Tests to check against when performance is gained from usage
/// of portable simd structures (_these tests were eventually removed_).
#[cfg(test)]
mod tests {
    use std::{hint::black_box, ops::Add, simd::num::SimdFloat};

    use ditherpunker_macros::simd_targets;
    use rand::Rng;

    extern crate test;

    const BUF_SIZE: usize = 250000;

    fn fill_mut_slice(buf: &mut [f32]) {
        let mut rng: rand::prelude::ThreadRng = rand::rng();
        buf.iter_mut()
            .for_each(|dst| *dst = rng.random_range(-1.0f32..1.0f32));
    }

    /// multiversioned impl with portable simd usage
    #[simd_targets]
    fn generic_simd_impl(buf: &[f32]) -> (f32, (usize, usize, usize), usize, usize) {
        use crate::simd_width;

        // compile time usize, can be used for generic structures in this impl:
        const LANES: usize = simd_width!(f32);

        let (prefix, simd, suffix) = buf.as_simd::<LANES>();
        let lens = (prefix.len(), simd.len(), suffix.len());

        let simd = simd.iter().fold(
            std::simd::Simd::<f32, LANES>::splat(0.0),
            std::simd::Simd::<f32, LANES>::add,
        );
        let simd_width = simd.len();
        let simd = simd.reduce_sum();
        let prefix: f32 = prefix.iter().sum();
        let suffix: f32 = suffix.iter().sum();

        (suffix + simd + prefix, lens, LANES, simd_width)
    }

    /// big buffers appear to be aligned
    #[test]
    fn test_generic_simd_align_normal() {
        let mut buf = vec![0.0f32; BUF_SIZE];
        fill_mut_slice(&mut buf);
        let (_, (suffix, simd, prefix), lanes, runtime_lanes) = generic_simd_impl(&buf);
        println!("not aligned: {} {}", suffix, prefix);
        println!("simd aligned: {}", simd);
        println!("lanes: {}", lanes);
        println!("runtime lanes: {}", runtime_lanes);
        assert_eq!(lanes, runtime_lanes);
    }

    /// forcing the buffer to be unaligned does indeed
    /// change the shape of .as_simd
    #[test]
    fn test_generic_simd_align_normal_force_unaligned() {
        let mut buf = vec![0.0f32; BUF_SIZE];
        fill_mut_slice(&mut buf);
        let (_, (suffix, simd, prefix), lanes, runtime_lanes) = generic_simd_impl(&buf[5..]);
        println!("not aligned: {} {}", suffix, prefix);
        println!("simd aligned: {}", simd);
        println!("lanes: {}", lanes);
        println!("runtime lanes: {}", runtime_lanes);
        assert_eq!(lanes, runtime_lanes);
    }

    #[bench]
    fn bench_generic_simd_impl(b: &mut test::Bencher) {
        let mut buf = vec![0.0f32; BUF_SIZE];
        fill_mut_slice(&mut buf);
        b.iter(|| {
            let sum = generic_simd_impl(&buf);
            black_box(sum)
        });
    }

    /// forcing the buffer to be unaligned does not
    /// decrese performance significantly in this case
    #[bench]
    fn bench_generic_simd_impl_force_unaligned(b: &mut test::Bencher) {
        let mut buf = vec![0.0f32; BUF_SIZE];
        fill_mut_slice(&mut buf);
        b.iter(|| {
            let sum = generic_simd_impl(&buf[5..]);
            black_box(sum)
        });
    }
}
