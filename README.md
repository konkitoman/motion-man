# Motion Man :construction:

This project was inspired by [Motion Canvas](https://github.com/motion-canvas/motion-canvas)

This wants to be a video maker, you build you video from code!

The experiment was a success!

## :warning:WIP :warning:

I don't know if i will continue this project!

## Currently

You cannot rewind media or go backwords, you can skip with `media.next()`

[x] video/audio decoding

[x] audio - is working but has problems
[x] video - is working but has problems

[] exports / video encoding

## Running

You need to have rust 1.70 installed and ffmpeg/libav
FFMPEG is only used for video/audio decoding!

You can read the `src/main.rs`

By default if you have a video.mkv in the current directory where you run `cargo run`
That video will play with the video stream 0 and audio stream 0, the application will wait until the video ends
The engine fps, sample rate and channels need to be the same as the video, the video is not converted!
Is possible to have audio problems!
Or the video to be to fast or to slow!
