```
Usage: equirect2cubemap [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>   the input equirectangular image's path
  <OUTPUT>  the directory to put the output images in, creates if doesn't exist

Options:
  -f, --format <FORMAT>                the image format of the output images [default: png] [possible values: jpg, png, webp]
  -i, --interpolation <INTERPOLATION>  interpolation used when sampling source image [default: linear] [possible values: linear, nearest]
  -s, --size <SIZE>                    size (px) of the output images, width = height [default: 512]
  -r, --rotate                         rotate to a z-up skybox if you use it in a y-up renderer
  -h, --help                           Print help
```
