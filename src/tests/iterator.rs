#[cfg(test)]
mod iterator_tests {
    use std::sync::Mutex;

    use rayon::iter::{IndexedParallelIterator, ParallelIterator};

    use crate::{
        tests::utils::BENCH_IMAGE_SIZE,
        utils::iterator::{GridIterator, ParGridIterator},
    };

    fn missing_pixel(data: &[bool]) -> bool {
        let missed_pixel = data.iter().enumerate().find_map(|(idx, val)| match *val {
            true => None,
            false => Some(idx),
        });
        if let Some(pixel) = missed_pixel {
            let x = pixel % BENCH_IMAGE_SIZE;
            let y = pixel / BENCH_IMAGE_SIZE;
            println!("missed pixel {}: {} {}", pixel, x, y);
        }
        missed_pixel.is_some()
    }

    #[test]
    fn test_grid_iterator_visits_indices() {
        let mut visit = vec![false; BENCH_IMAGE_SIZE * BENCH_IMAGE_SIZE];
        for (x, y, idx) in GridIterator::new(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE) {
            assert_eq!(
                BENCH_IMAGE_SIZE * y + x,
                idx,
                "1D pixel index missmatches 2D coordinates"
            );
            assert!(!visit[idx], "Pixel visited twice");
            visit[idx] = true;
        }
        assert!(!missing_pixel(&visit), "Pixel not visited");
    }

    #[test]
    fn test_par_grid_iterator_visits_indices() {
        let mut visit_data = vec![false; BENCH_IMAGE_SIZE * BENCH_IMAGE_SIZE];
        let visit_mutex = Mutex::new(visit_data.as_mut_slice());
        ParGridIterator::new(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE)
            .par_iter()
            .for_each(|(x, y, idx)| {
                assert_eq!(
                    BENCH_IMAGE_SIZE * y + x,
                    idx,
                    "1D pixel index missmatches 2D coordinates"
                );
                let mut visit = visit_mutex.lock().unwrap();
                assert!(!visit[idx], "Pixel visited twice");
                visit[idx] = true;
            });
        assert!(!missing_pixel(&visit_data), "Pixel not visited");
    }

    #[test]
    fn test_par_grid_iterator_visits_chunked_indices() {
        let mut visit_data = vec![false; BENCH_IMAGE_SIZE * BENCH_IMAGE_SIZE];
        let visit_mutex = Mutex::new(visit_data.as_mut_slice());
        ParGridIterator::new(BENCH_IMAGE_SIZE, BENCH_IMAGE_SIZE)
            .par_iter()
            .fold_chunks_with(BENCH_IMAGE_SIZE, (), |_, (x, y, idx)| {
                assert_eq!(
                    BENCH_IMAGE_SIZE * y + x,
                    idx,
                    "1D pixel index missmatches 2D coordinates"
                );
                let mut visit = visit_mutex.lock().unwrap();
                assert!(!visit[idx], "Pixel visited twice");
                visit[idx] = true;
            })
            .for_each(|_| {});
        assert!(!missing_pixel(&visit_data), "Pixel not visited");
    }
}
