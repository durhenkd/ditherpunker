pub const FLOYD_STEINBERG: [f32; 6] = [0.0, -1.0, 0.4375, 0.1875, 0.3125, 0.0625];
pub const FLOYD_STEINBERG_SIZE: [usize; 2] = [3, 2];

pub const JARVIS_JUDICE_NINKE: [f32; 15] = [
    0.0, 0.0, -1.0, 0.14583333, 0.10416666, 0.0625, 0.10416666, 0.14583333, 0.10416666, 0.0625,
    0.02083333, 0.0625, 0.10416666, 0.0625, 0.02083333,
];
pub const JARVIS_JUDICE_NINKE_SIZE: [usize; 2] = [5, 3];

pub const ATKINSON: [f32; 12] = [
    0.0, -1.0, 0.125, 0.125, 0.125, 0.125, 0.125, 0.0, 0.0, 0.125, 0.0, 0.0,
];
pub const ATKINSON_SIZE: [usize; 2] = [4, 3];
