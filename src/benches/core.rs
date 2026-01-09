#[cfg(test)]
mod core_benches {
    use rand::{Rng, rngs::ThreadRng};

    use crate::utils::buffer::uninitialized_buffer;

    extern crate test;

    fn sum(slice: &Vec<usize>) -> usize {
        let mut sum: usize = 0;
        for item in slice {
            sum += item;
        }
        sum
    }

    fn create_vec_with_default_macro(default: usize, size: usize) -> Vec<usize> {
        vec![default; size]
    }

    fn create_vec_with_default_uninitialized(default: usize, size: usize) -> Vec<usize> {
        let mut data = unsafe { uninitialized_buffer::<usize>(size) };
        for i in 0..size {
            data[i] = default;
        }
        data
    }

    fn fill_vec_with_random(data: &mut [usize], rng: &mut ThreadRng) {
        for index in 0..data.len() {
            let value: f32 = rng.random();
            let value = (value * 100.0) as usize;
            data[index] = value;
        }
    }

    #[bench]
    fn bench_vec_creation(b: &mut test::Bencher) {
        let mut rng = rand::rng();
        b.iter(|| {
            let mut data = create_vec_with_default_macro(0, 100);
            fill_vec_with_random(&mut data, &mut rng);
            sum(&data);
        });
    }

    #[bench]
    fn bench_uninitialized_buffer_creation(b: &mut test::Bencher) {
        let mut rng = rand::rng();
        b.iter(|| {
            let mut data = create_vec_with_default_uninitialized(0, 100);
            fill_vec_with_random(&mut data, &mut rng);
            sum(&data);
        });
    }

    #[bench]
    fn bench_uninitialized_buffer_creation_no_default(b: &mut test::Bencher) {
        let mut rng = rand::rng();
        b.iter(|| {
            let mut data = unsafe { uninitialized_buffer::<usize>(100) };
            fill_vec_with_random(&mut data, &mut rng);
            sum(&data);
        });
    }
}
