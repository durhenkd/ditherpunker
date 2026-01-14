use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ditherpunker::dithering::threshold::threshold_transform::ThresholdImpl;

pub(crate) mod threshold_utils;
pub(crate) mod utils;

use threshold_utils::*;
use utils::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bayer_0_transform_fit");

    macro_rules! benchmark_by_param {
        ($colors:expr, $strategy:expr) => {{
            let (input, output) = std::hint::black_box(threshold_data(BENCH_IMAGE_SIZE));
            for colors in $colors {
                let cfg = threshold_config(colors);
                let strategy = $strategy(colors);
                // .to_string also includes implementation details, such as lanes used
                let id = BenchmarkId::new(strategy.to_string(), cfg.map_size());
                benchmark_threshold_strategy(
                    &mut group,
                    id,
                    cfg,
                    $strategy(colors),
                    input.clone(),
                    output.clone(),
                );
            }
        }};
    }

    let fixed_lanes = [2, 4, 8];
    let fit_colors = [3, 5, 6, 7, 10, 12, 20, 24, 30];
    benchmark_by_param!(fit_colors, |_| ThresholdImpl::Scalar);
    for lanes in fixed_lanes {
        benchmark_by_param!(fit_colors, |_| ThresholdImpl::Simd { lanes });
    }

    group.finish();
}

criterion_group!(bayer_transform_fit_strategy, criterion_benchmark);
criterion_main!(bayer_transform_fit_strategy);
