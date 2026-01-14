#[cfg(test)]
mod bayer_strategy_tests {
    use itertools::Itertools;

    use crate::{
        dithering::threshold::{
            matrices,
            threshold_transform::{ThresholdConfig, ThresholdImpl},
        },
        tests::utils::*,
        texture::{Texture, TextureRef},
        transform::prelude::*,
        utils::pixel::RGB,
    };

    /// Test image size - kept small for fast tests
    const TEST_SIZE: usize = 100;

    /// Generate test data: grayscale input and RGB output textures
    fn test_data(size: usize) -> (Texture<f32>, Texture<RGB>) {
        let rgb_data = gen_random_image(size);
        let grayscale: Vec<f32> = rgb_data.iter().map(|pixel| pixel.grayscale()).collect_vec();

        let input = Texture::from_slice(size as u32, size as u32, 1, &grayscale);
        let output = Texture::new(size as u32, size as u32, 1);

        (input, output)
    }

    /// Create a config with specified color count
    fn create_config(colors: usize) -> ThresholdConfig {
        let color_map = if colors == 2 {
            default_color_map()
        } else {
            random_color_map(colors)
        };
        ThresholdConfig::new(1, matrices::BAYER0.to_vec(), color_map)
    }

    /// Assert that two images match pixel by pixel
    fn assert_images_match(a: &[RGB], b: &[RGB], width: usize, strategy_a: &str, strategy_b: &str) {
        assert_eq!(a.len(), b.len(), "image lengths don't match");
        for (idx, (a_pixel, b_pixel)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(
                a_pixel,
                b_pixel,
                "Pixel mismatch at index {} (x={}, y={}): {}={:?}, {}={:?}",
                idx,
                idx % width,
                idx / width,
                strategy_a,
                a_pixel,
                strategy_b,
                b_pixel
            );
        }
    }

    /// Apply a strategy to the given input/output textures
    fn apply_strategy(
        strategy: ThresholdImpl,
        config: ThresholdConfig,
        input: &Texture<f32>,
        output: &mut Texture<RGB>,
    ) {
        let mut transform = strategy.build(config);
        transform.prepare(input.shape(), output.shape());
        transform.apply(input.as_texture_slice(), output.as_texture_mut_slice());
    }

    /// Macro to generate BayerStrategy comparison tests
    macro_rules! test_strategy_comparison {
        ($test_name:ident, $strategy_a:expr, $strategy_b:expr, $colors:expr, $label_a:expr, $label_b:expr) => {
            #[test]
            fn $test_name() {
                let (input, mut output_a) = test_data(TEST_SIZE);
                let mut output_b = Texture::new(TEST_SIZE as u32, TEST_SIZE as u32, 1);

                let config = create_config($colors);

                apply_strategy($strategy_a, config.clone(), &input, &mut output_a);
                apply_strategy($strategy_b, config, &input, &mut output_b);

                assert_images_match(
                    output_a.as_ref(),
                    output_b.as_ref(),
                    TEST_SIZE,
                    $label_a,
                    $label_b,
                );
            }
        };
    }

    // Tests for 2 colors (power of 2)

    test_strategy_comparison!(
        test_scalar_vs_simd_fixed_2_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::SimdFixed { lanes: 2 },
        2,
        "scalar",
        "simd-fixed-2"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_2_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 2 },
        2,
        "scalar",
        "simd-fit-2"
    );

    // Tests for 4 colors (power of 2)

    test_strategy_comparison!(
        test_scalar_vs_simd_fixed_4_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::SimdFixed { lanes: 4 },
        4,
        "scalar",
        "simd-fixed-4"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_4_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 4 },
        4,
        "scalar",
        "simd-fit-4"
    );

    // Tests for 8 colors (power of 2)

    test_strategy_comparison!(
        test_scalar_vs_simd_fixed_8_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::SimdFixed { lanes: 8 },
        8,
        "scalar",
        "simd-fixed-8"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_8_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 8 },
        8,
        "scalar",
        "simd-fit-8"
    );

    // Tests for 16 colors (power of 2)

    test_strategy_comparison!(
        test_scalar_vs_simd_fixed_16_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::SimdFixed { lanes: 16 },
        16,
        "scalar",
        "simd-fixed-16"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_16_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 16 },
        16,
        "scalar",
        "simd-fit-16"
    );

    // Tests for non-power-of-2 colors (testing Fit strategies)

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_3_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 2 },
        3,
        "scalar",
        "simd-fit-2-3colors"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_5_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 4 },
        5,
        "scalar",
        "simd-fit-4-5colors"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_7_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 4 },
        7,
        "scalar",
        "simd-fit-4-7colors"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_12_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 8 },
        12,
        "scalar",
        "simd-fit-8-12colors"
    );

    // Additional edge case tests

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_24_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 16 },
        24,
        "scalar",
        "simd-fit-16-24colors"
    );

    test_strategy_comparison!(
        test_scalar_vs_simd_fit_32_colors,
        ThresholdImpl::Scalar,
        ThresholdImpl::Simd { lanes: 16 },
        32,
        "scalar",
        "simd-fit-16-32colors"
    );
}
