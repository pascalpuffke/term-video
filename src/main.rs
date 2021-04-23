/*
   This file is part of term-video.

   term-video is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   term-video is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with term-video.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::{AppSettings, Clap};
use image::{io::Reader, DynamicImage, GenericImageView, Pixel};
use std::{
    fs,
    process::{exit, Command, Stdio},
    str::FromStr,
    thread,
    time::Duration,
};
use walkdir::WalkDir;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Pascal Puffke <pascal@pascalpuffke.de>", setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(
        short,
        long,
        default_value = "split_frames",
        about = "Where to save temporary frame data"
    )]
    cache: String,
    #[clap(
        short,
        long,
        about = "Input video file, can be any format as long as it's supported by ffmpeg."
    )]
    input: String,
    #[clap(
        short,
        long,
        about = "Horizontal playback resolution [default: current terminal rows]"
    )]
    width: Option<u32>,
    #[clap(
        short,
        long,
        about = "Vertical playback resolution [default: current terminal columns]"
    )]
    height: Option<u32>,
    #[clap(
        short,
        long,
        about = "Playback frame rate [default: input video FPS, or 30 should ffprobe fail]"
    )]
    fps: Option<u32>,
}

fn main() {
    let opts = Opts::parse();
    let term_dim = term_size::dimensions().unwrap_or((80, 24));
    let w = opts.width.unwrap_or(term_dim.0 as u32);
    let h = opts.height.unwrap_or(term_dim.1 as u32);
    let fps = opts
        .fps
        .unwrap_or(get_frame_rate(&opts.input).unwrap_or(30));

    make_dir(&opts.cache);
    split_and_resize_frames(&opts.input, &opts.cache, w, h);
    display_loop(&opts.cache, w, h, fps);

    // clean up temporary directory before exiting
    fs::remove_dir_all(&opts.cache).expect("could not delete temporary directory, enjoy the mess");

    println!("Finished playback at {} fps", fps);
}

fn make_dir(name: &str) {
    if let Err(_) = fs::create_dir(name) {
        fs::remove_dir_all(name).expect(&format!("could not delete directory {}", name));
        fs::create_dir(name).expect(&format!("could not create directory {}", name));
    }
}

fn split_and_resize_frames(file_name: &str, cache_dir: &str, width: u32, height: u32) {
    // ffmpeg -i <file_name> -f image2 -vf scale=<w:h> <cache>/frame-%07d.png
    Command::new("ffmpeg")
        .args(vec![
            "-i",
            file_name,
            "-f",
            "image2",
            "-vf",
            &format!("scale={}:{}", width, height),
            &format!("{}/frame-%07d.png", cache_dir),
        ])
        .stdout(Stdio::null())
        .output()
        .unwrap_or_else(|e| {
            println!("Failed to execute ffmpeg - do you have it installed? {}", e);
            exit(1);
        });
}

fn get_frame_rate(video: &str) -> Option<u32> {
    let ffprobe = Command::new("ffprobe")
        .args(vec![
            "-v",
            "error",
            "-select-streams",
            "-v:0",
            "-show_entries",
            "stream=r_frame_rate",
            "-of",
            "csv=s=x:p=0",
            video,
        ])
        .output();

    if let Ok(out) = ffprobe {
        if let Ok(parsed) = u32::from_str(
            String::from_utf8_lossy(&out.stdout)
                .to_string()
                .split('/')
                .nth(0)
                .expect("Error parsing ffprobe output string"),
        ) {
            return Some(parsed);
        }
    }

    None
}

fn display_loop(cache_dir: &str, width: u32, height: u32, frame_rate: u32) {
    // unwrapunwrapunwrapunwrapunwrapunwrapunwrapunwrap
    let frames = WalkDir::new(cache_dir)
        .sort_by_file_name()
        .into_iter()
        .skip(1)
        .map(|f| {
            Reader::open(format!(
                "{}/{}",
                cache_dir,
                f.unwrap().file_name().to_str().unwrap()
            ))
            .unwrap()
            .decode()
            .unwrap()
        })
        .collect::<Vec<DynamicImage>>();
    let mut frame_buffer = String::new();
    let mut display_buffer: Vec<String> = Vec::with_capacity(frames.len());

    // Filling the display buffer before starting playback. Will use more memory and takes a lot
    // longer to get started, but eliminates artifacts
    for frame in frames {
        for y in 0..height {
            for x in 0..width {
                let luma = frame.get_pixel(x, y).to_luma();
                let pixel = match *luma.0.get(0).unwrap() {
                    // It can't be that bad if it works, right?
                    0 => ' ',
                    1..=21 => '.',
                    22..=43 => ',',
                    44..=65 => '-',
                    66..=87 => '~',
                    88..=109 => ':',
                    110..=131 => ';',
                    132..=153 => '=',
                    154..=175 => '!',
                    176..=197 => '*',
                    198..=219 => '#',
                    220..=241 => '$',
                    _ => '@',
                };

                frame_buffer.push(pixel);
            }

            frame_buffer.push('\n');
        }

        display_buffer.push(frame_buffer.clone());
        frame_buffer.clear();
    }

    clear_screen();

    // Displaying each frame
    for frame in &display_buffer {
        println!("{}", frame);

        thread::sleep(Duration::from_micros((1000000 / frame_rate) as u64));
        clear_screen();
    }
}

fn clear_screen() {
    print!("{}[2J", 27 as char);
}
