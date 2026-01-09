use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ditherpunker::dithering::threshold::bayer_transform::BayerStrategy;

pub(crate) mod bayer_transform_utils;
use bayer_transform_utils::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bayer_0_transform");

    macro_rules! benchmark_by_param {
        ($colors:expr, $strategy:expr) => {
            for colors in $colors {
                let cfg = config(colors);
                let strategy = $strategy(colors);
                let id = BenchmarkId::new(strategy.name(), cfg.colors_len());
                benchmark_strategy(&mut group, id, cfg, $strategy(colors), BENCH_IMAGE_SIZE);
            }
        };
    }

    let fixed_lanes = [2, 4, 8, 16];
    benchmark_by_param!(fixed_lanes, |_| BayerStrategy::Scalar);
    benchmark_by_param!(fixed_lanes, |lanes| BayerStrategy::SimdFixed { lanes });
    benchmark_by_param!(fixed_lanes, |lanes| BayerStrategy::Simd { lanes });

    BayerStrategy::Scalar.to_string();

    group.finish();
}

criterion_group!(bayer_transform_strategy, criterion_benchmark);
criterion_main!(bayer_transform_strategy);
