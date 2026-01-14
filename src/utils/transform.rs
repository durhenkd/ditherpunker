use crate::utils::buffer;

/// Precompute the result of a tilable computation
/// for faster memory access by row.
///
/// > A(x, y) * B(n, m) -> C(x, m)
#[inline(always)]
pub fn precompute_tiled_rows<T, MapFn>(tile_size: usize, row_size: usize, map: MapFn) -> Vec<T>
where
    MapFn: Fn(usize, usize, usize) -> T,
{
    // SAFETY: buffer is init by map fn
    let mut cache = unsafe { buffer::uninitialized_buffer(tile_size * row_size) };
    let mut idx = 0;
    for y in 0..tile_size {
        for x in 0..row_size {
            cache[idx] = map(x, y, idx);
            idx += 1;
        }
    }
    cache
}

#[cfg(test)]
mod tests {
    use super::precompute_tiled_rows;

    #[test]
    fn test_precompute_tiled_rows_simple() {
        let buf = precompute_tiled_rows(5, 10, |x, y, idx| (x, y, idx));
        assert_eq!(buf.len(), 5 * 10);

        let mut idx = 0;
        for y in 0..5 {
            for x in 0..10 {
                assert_eq!(buf[idx], (x, y, idx));
                idx += 1;
            }
        }
    }
}

#[cfg(test)]
mod benches {
    use super::*;

    extern crate test;

    // Simulates a typical use case: looking up values from a small tiled pattern
    const TILE_SIZE: usize = 8;
    const ROW_SIZE: usize = 1024;
    const PATTERN: [f32; 64] = [
        0.0, 32.0, 8.0, 40.0, 2.0, 34.0, 10.0, 42.0, 48.0, 16.0, 56.0, 24.0, 50.0, 18.0, 58.0,
        26.0, 12.0, 44.0, 4.0, 36.0, 14.0, 46.0, 6.0, 38.0, 60.0, 28.0, 52.0, 20.0, 62.0, 30.0,
        54.0, 22.0, 3.0, 35.0, 11.0, 43.0, 1.0, 33.0, 9.0, 41.0, 51.0, 19.0, 59.0, 27.0, 49.0,
        17.0, 57.0, 25.0, 15.0, 47.0, 7.0, 39.0, 13.0, 45.0, 5.0, 37.0, 63.0, 31.0, 55.0, 23.0,
        61.0, 29.0, 53.0, 21.0,
    ];

    #[bench]
    fn bench_precompute_tiled_rows_generic(b: &mut test::Bencher) {
        let pattern = PATTERN;
        b.iter(|| {
            let cache = precompute_tiled_rows(
                TILE_SIZE,
                ROW_SIZE,
                #[inline(always)]
                |x, y, _| pattern[y * TILE_SIZE + (x % TILE_SIZE)],
            );
            test::black_box(cache)
        });
    }

    #[bench]
    fn bench_precompute_tiled_rows_inlined(b: &mut test::Bencher) {
        let pattern = PATTERN;
        b.iter(|| {
            let mut cache = unsafe { buffer::uninitialized_buffer::<f32>(TILE_SIZE * ROW_SIZE) };
            let mut idx = 0;
            for y in 0..TILE_SIZE {
                for x in 0..ROW_SIZE {
                    cache[idx] = pattern[y * TILE_SIZE + (x % TILE_SIZE)];
                    idx += 1;
                }
            }
            test::black_box(cache)
        });
    }
}
