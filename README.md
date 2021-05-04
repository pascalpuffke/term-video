# term-video
![Preview](preview.png)
I guess this is usable now...

# Compilation
Since this project is built using Rust, install its toolchain first, for example using [rustup](https://rustup.rs/).

```
git clone https://github.com/pascalpuffke/term-video
cd term-video
cargo build --release
```

After the build process has finished, the executable is located under ```target/release/term-video```. You can then copy it to a location that is checked by PATH:

```cp target/release/term-video /usr/bin```

# Usage
```term-video [OPTIONS] --input <input>```

Options:

```--help```: Prints help information

```-V```, ```--version```: Prints version information

```-c```, ```--cache <cache>```: Where to save temporary frame data [default: split_frames]

```-f```, ```--fps <fps>```: Playback frame rate [default: input video FPS, or 30 should ffprobe fail]

```-h```, ```--height <height>```: Vertical playback resolution [default: current terminal columns]

```-i```, ```--input <input>```: Input video file, can be any format as long as it's supported by ffmpeg.

```-w```, ```--width <width>```: Horizontal playback resolution [default: current terminal rows]

# Dependencies

**Runtime dependencies**:

[ffmpeg, ffprobe](https://ffmpeg.org/)

**Build dependencies**:

[image-rs](https://github.com/image-rs/image) 0.23.14

[walkdir](https://github.com/BurntSushi/walkdir) 2.3.2

[clap](https://github.com/clap-rs/clap) 3.0.0-beta.2
