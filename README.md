# Ditherpunker

This is an image editing software which aims to turns regular photos into [ditherpunk illustrations](https://www.reddit.com/r/ditherpunk/).

## What is ditherpunk?

Ditherpunk is a pixel art style which is charactized by low number of colors (often 1-bit, black & white) making the use of dithering necessary to create the illusion of shading. There are a couple of indie games making fantastic use of this aesthetic, like [Return of the Obra Dinn](https://store.steampowered.com/app/653530/Return_of_the_Obra_Dinn/) and [Critters for Sale](https://store.steampowered.com/app/1078420/Critters_for_Sale/).

## How to use this

This repo doesn't have a release yet, just install the rust toolchain, use your favourite shell, and:

```
cargo run -- input/file/path.png output/file/path.png path_to_config.json
```

_NOTE: currently only PNG format is available as output, regardless of what extension you use in your path._

### Config file:

The program needs a config file to know how to edit your images:

```js
{
  "processing_width": 300, // max width at which all the processing is done (ratio is preserved)
  "processing_height": 300, // max height at which all the processing is done (ratio is preserved)
  "brigthness_delta": 30, // increase/decrease brightness before dithering
  "constrast_delta": 30, // increase/decrease contrast before dithering
  "dithering_type": "blue_noise", // the dithering technique used, see list
  "color_map": [ // optional field: custom list of colors used for dithering,  (default is black and white)
    {
      "color": "#101010", // color in hex, the '#' at the beggining is optional
      "magnitude": 0.8 // optional field: affects the amount of color used
    },
    {
      "color": "0000aa", // doesn't matter if the letters are in uppercase or lowercase
      "offset": 0.15, // optional field: bias for lighter/darker color
      "magnitude": 0.85
    },
    {
      "color": "10F022",
      "offset": 0.50,
      "magnitude": 0.5
    },
    "f0f0f0" // shorthand for when not using offset or magnitude
  ],
  "output_scale": 4 // scale the image before writing it (done to preserve the pixel effect)
}
```

### List of dithering techniques

- `rand` - pure randomness, works better with bigger processing sizes
- `bayer_0` - Bayer(0) 2x2 matrix, clear patterns
- `bayer_1` - Bayer(1) 4x4 matrix, clear patterns
- `bayer_2` - Bayer(2) 8x8 matrix, can leave some unpleasing artefacts on the image
- `bayer_3` - Bayer(3) 16x16 matrix, can leave some unpleasing artefacts on the image
- `blue_noise` - uses a pre-computed 128x128 blue noise texture
- `atkinson` - error-diffusion with the Atkinson matrix (**_NOT IMPLEMENTED_**)
- `jarvis` - error-diffusion with the Jarvis-Judice-Ninke matrix (**_NOT IMPLEMENTED_**)
- `floyd` - error-diffusion with the Floyd-Steinberg matrix (**_NOT IMPLEMENTED_**)
