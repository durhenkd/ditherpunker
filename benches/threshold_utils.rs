use criterion::{BenchmarkGroup, BenchmarkId, measurement::WallTime};
use ditherpunker::{
    dithering::threshold::threshold_transform::{ThresholdConfig, ThresholdImpl},
    prelude::{RGB, TextureTransform},
    texture::{Texture, TextureRef},
};

/// Run threshold strategy under a criterion group
pub fn benchmark_threshold_strategy(
    group: &mut BenchmarkGroup<'_, WallTime>,
    id: BenchmarkId,
    config: ThresholdConfig,
    strategy: ThresholdImpl,
    input: Texture<f32>,
    mut output: Texture<RGB>,
) {
    let mut transform = strategy.build(config);
    transform.prepare(input.shape(), output.shape());

    group.bench_with_input(id, &strategy, |b, _| {
        b.iter(|| {
            transform.apply(input.as_texture_slice(), output.as_texture_mut_slice());
        });
    });
}
