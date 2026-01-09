use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ditherpunker::dithering::threshold::bayer_transform::BayerStrategy;

pub(crate) mod bayer_transform_utils;
use bayer_transform_utils::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bayer_0_transform_par_fit");

    macro_rules! benchmark_by_param {
        ($colors:expr, $strategy:expr) => {
            for colors in $colors {
                let cfg = config(colors);
                let strategy = $strategy(colors);
                // to_string also includes implementation details, such as lanes used
                let id = BenchmarkId::new(strategy.to_string(), cfg.colors_len());
                benchmark_strategy(&mut group, id, cfg, $strategy(colors), BENCH_IMAGE_SIZE);
            }
        };
    }

    let fixed_lanes = [2, 4, 8];
    let fit_colors = [3, 5, 6, 7, 10, 12, 20, 24, 30];
    benchmark_by_param!(fit_colors, |_| BayerStrategy::ScalarPar);
    for lanes in fixed_lanes {
        benchmark_by_param!(fit_colors, |_| BayerStrategy::SimdPar { lanes });
    }

    group.finish();
}

criterion_group!(bayer_transform_par_fit_strategy, criterion_benchmark);
criterion_main!(bayer_transform_par_fit_strategy);
