use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Applies default SIMD target configurations for ditherpunker.
///
/// This macro expands to `#[multiversion(targets(...))]`
///
/// # Example
///
/// ```
/// use ditherpunker_macros::simd_targets;
///
/// #[simd_targets]
/// pub fn my_simd_function(x: f32) -> f32 {
///     x * 2.0
/// }
/// ```
#[proc_macro_attribute]
pub fn simd_targets(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);

    // Use minimal targets for debug builds to speed up compilation
    #[cfg(debug_assertions)]
    let expanded = quote! {
        #[multiversion::multiversion(targets(
            "x86_64+sse2",
            "aarch64+neon",
        ))]
        #func
    };

    // Use full target set for release builds
    #[cfg(not(debug_assertions))]
    let expanded = quote! {
        #[multiversion::multiversion(targets(
            // x86_64 (most modern desktops/servers)
            "x86_64+avx512f+avx512bw+avx512cd+avx512dq+avx512vl",
            "x86_64+avx2+fma",
            "x86_64+sse4.2",
            "x86_64+sse2",  // baseline for all x86_64

            // x86 32-bit (legacy)
            // "x86+sse2",

            // ARM64 (mobile, Apple Silicon, ARM servers)
            "aarch64+neon+sve", // newer ARM servers/CPUs
            "aarch64+neon",     // baseline for all aarch64

            // RISC-V (emerging, optional)
            // "riscv64+v",  // vector extension
        ))]
        #func
    };

    TokenStream::from(expanded)
}
