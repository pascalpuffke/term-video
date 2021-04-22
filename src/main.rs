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

use image::{io::Reader, DynamicImage, GenericImageView, Pixel};
use std::{
    env, fs,
    process::{Command, Stdio},
    str::FromStr,
    thread,
    time::Duration,
};
use walkdir::WalkDir;

// TODO investigate playback speed issues; might just be the terminal emulator at this point

// TODO replace with structopt or clap
const FRAMES_DIR: &str = "split_frames";
const WIDTH: u32 = 132;
const HEIGHT: u32 = 43;
const FPS: u32 = 30;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = args.get(1).expect("need to specify an input video file");

    println!("Using file {}", input);

    let mut frame_rate = FPS;
    let mut width = WIDTH;
    let mut height = HEIGHT;

    if let Some(fps) = get_frame_rate(input) {
        frame_rate = fps;
    } // fall back to 30fps otherwise

    if let Some((w, h)) = term_size::dimensions() {
        width = w as u32;
        height = h as u32;
    }

    make_dir();
    split_and_resize_frames(input, FRAMES_DIR, width, height);
    display_loop(FRAMES_DIR, width, height, frame_rate);

    // clean up temporary directory before exiting
    fs::remove_dir_all(FRAMES_DIR).expect("could not delete temporary directory, enjoy the mess");

    println!("Finished playback at {} fps", frame_rate);
}

fn make_dir() {
    // Creating directory to store all frames in
    if let Err(_) = fs::create_dir(FRAMES_DIR) {
        // delete and re-create it
        fs::remove_dir_all(FRAMES_DIR)
            .expect(&format!("could not delete directory {}", FRAMES_DIR));
        fs::create_dir(FRAMES_DIR).expect(&format!("could not create directory {}", FRAMES_DIR));
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
        .expect("failed to execute ffmpeg");
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
    // Terribly bad code.
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
    let num_frames = frames.len();
    let mut frame_buffer = String::new();
    let mut display_buffer: Vec<String> = Vec::with_capacity(num_frames);

    // Filling the frame buffer before starting playback. Will use more memory and takes a lot
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
