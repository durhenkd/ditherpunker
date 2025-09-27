#[derive(Debug, Clone, Copy)]
pub enum ErrorDiffusionType {
    FloydSteinberg,
    JarvisJudiceNinke,
    Atkinson,
}

// TODO: implement the error diffusion algorithms
