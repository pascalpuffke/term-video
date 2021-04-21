use image::{imageops::FilterType, io::Reader, GenericImageView, Pixel};
use std::{
    env, fs,
    process::{Command, Stdio},
    thread, time,
    time::Duration,
};
use walkdir::{DirEntry, WalkDir};

// Global constants, feel free to mess around with them.
const FRAMES_DIR: &str = "split_frames";
// TODO get terminal dimensions using term-size crate
const WIDTH: u32 = 132;
const HEIGHT: u32 = 43;
// TODO get video framerate automatically, something something ffmpeg i dont know
const FPS: u32 = 30;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = args.get(1).expect("need to specify an input video file");

    /*
    if !input.ends_with(".mp4") {
        panic!("not a video file: {}", input);
    }
     */

    println!("using video file {}", input);

    make_dir();
    split_frames(input);

    println!("split video file into its separate frames");

    display_loop();

    // clean up temporary directory before exiting
    fs::remove_dir_all(FRAMES_DIR)
        .expect("could not delete temporary directory, have fun with the mess");
}

fn make_dir() {
    // Creating directory to store all frames in
    if let Err(_) = fs::create_dir(FRAMES_DIR) {
        println!("directory {} already exists, deleting it", FRAMES_DIR);

        // delete and re-create it
        fs::remove_dir_all(FRAMES_DIR)
            .expect(&format!("could not delete directory {}", FRAMES_DIR));
        fs::create_dir(FRAMES_DIR).expect(&format!("could not create directory {}", FRAMES_DIR));
    } else {
        println!("created directory {}", FRAMES_DIR)
    }
}

fn split_frames(file_name: &str) {
    // ffmpeg -i $file.mp4 -f image2 $split_frames/frame-%07d.png
    Command::new("ffmpeg")
        .args(vec![
            "-i",
            file_name,
            "-f",
            "image2",
            &format!("{}/frame-%07d.png", FRAMES_DIR),
        ])
        .stdout(Stdio::null())
        .output()
        .expect("failed to execute ffmpeg");
}

// This might be the worst Rust code you will ever see.
fn display_loop() {
    let files = WalkDir::new(FRAMES_DIR)
        .sort_by_file_name()
        .into_iter()
        .skip(1)
        .map(|f| f.unwrap())
        .collect::<Vec<DirEntry>>();
    let mut resized_images = Vec::new();
    let mut time_since_last_frame = get_current_time_ms();

    for i in 0..files.len() {
        print!("processing frame {} of {}... ", i, files.len());

        let file = files.get(i).expect("failed to access file");
        let file_name = format!("{}/{}", FRAMES_DIR, file.file_name().to_str().unwrap());
        let image = Reader::open(file_name.clone())
            .unwrap()
            .decode()
            .unwrap()
            .resize_exact(WIDTH, HEIGHT, FilterType::Nearest);

        resized_images.push(image);
        fs::remove_file(&file_name)
            .expect(&format!("could not delete temporary file {}", file_name));

        println!("({}ms)", get_current_time_ms() - time_since_last_frame);
        time_since_last_frame = get_current_time_ms();
    }

    clear_screen();
    let mut buffer = String::new();
    for frame in resized_images {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let luma = frame.get_pixel(x, y).to_luma();
                let char = match *luma.0.get(0).unwrap() {
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

                buffer.push(char);
            }
            buffer.push('\n');
        }

        println!("{}", buffer);
        buffer.clear();

        thread::sleep(Duration::from_millis((1000 / FPS) as u64));

        clear_screen();
    }
}

fn clear_screen() {
    print!("{}[2J", 27 as char);
}

fn get_current_time_ms() -> u128 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
