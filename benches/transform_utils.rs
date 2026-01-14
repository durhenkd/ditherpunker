use std::{fmt::Display, hint::black_box};

use criterion::{BenchmarkGroup, BenchmarkId, measurement::WallTime};
use ditherpunker::{
    prelude::TextureTransform,
    texture::{Texture, TextureRef},
};

pub fn bench_transform<In, Out, T: Display>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    id: BenchmarkId,
    param: T,
    transform: &mut impl TextureTransform<Input = In, Output = Out>,
    input: Texture<In>,
    mut output: Texture<Out>,
) {
    group.bench_with_input(id, &param, |b, _| {
        transform.prepare(input.shape(), output.shape());
        b.iter(|| {
            let res = transform.apply(input.as_texture_slice(), output.as_texture_mut_slice());
            black_box(res);
        });
    });
}

pub fn bench_transform_by_param<In, Out, Param, GetName, GetData>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    transform: &mut impl TextureTransform<Input = In, Output = Out>,
    get_name: GetName,
    params: &[Param],
    get_data: GetData,
) where
    Param: Display,
    GetName: Fn(&Param) -> String,
    GetData: Fn(&Param) -> (Texture<In>, Texture<Out>),
{
    for param in params {
        let name = get_name(param);
        let (input, output) = get_data(param);
        bench_transform(
            group,
            BenchmarkId::new(name, param),
            param,
            transform,
            input,
            output,
        );
    }
}
