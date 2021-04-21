# term-video
Don't use this.

I really just wrote this program to play 'Bad Apple' in my terminal, be happy for 10 seconds that it actually sort of works and then return to the depressing reality.

Do not expect this project to grow into anything larger than that, though I might improve it here or there, depending on how I feel.

# Usage
```cargo run -- <path-to-video>```

Then enjoy your computer fan noises. It should not be too bad, as this program only utilizes a single thread.

# Problems
- It only utilizes a single thread and is therefore painfully slow. On my decently fast machine, it took 15 minutes for converting ~6700 frames to play back Bad Apple.

- It does not yet determine the terminal size automatically and uses hard-coded values.

- It does not yet detect the FPS of the input video and uses a hard-coded value.

- Playback is not frame-perfectly timed to the source. It was about 5 to 8 percent slower than the input video.

- It's utterly useless.

# Dependencies

[image-rs](https://github.com/image-rs/image) 0.23.14
[walkdir](https://github.com/BurntSushi/walkdir) 2.3.2