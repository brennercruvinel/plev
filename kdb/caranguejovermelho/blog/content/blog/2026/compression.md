+++
title = "AV1 for robotics AI streaming, training and storage"
date = 2026-01-01
draft = true
+++

# AV1 for robotics AI streaming, training and storage

**Published:** August 27, 2025
**Author:** Haixuan Tao
**Type:** Community Article

**Source:** https://huggingface.co/blog/haixuantao/av1-for-ai-and-robotics

---

## tldr: AV1 Performance Gains

**Streaming:** Streaming AV1 packet for rgb and depth is better than alternatives like .jpeg or .tiff

- 60% smaller files/bitrate for rgb+depth (0.32MB vs 0.75MB @ 720p) compared to closest alternatives
- 2x higher frame rates (25fps vs 12fps @ 720p)
- 2x lower latency for RGB+Depth (250ms vs 525ms @ 720p)

**Training:** Batch reading .avif files instead of loading .mp4 videos and retrieving random frames is way faster

- 4x faster than TorchCodec using single .avif image with a parallel image reader instead of videos
- Reading single .avif frame is a lot more simple than mp4 as it removes ffmpeg dependency
- Enable just-in-time dataset image downloading allowing to not wait videos download to start training

**Storage:** Replacing .mp4 videos with .avif single file add more granularity at the cost of more storage

- Enable more variety of dataset like continuous robotic data or streamed data that does not have clear start and end of episodes
- Enable frame metadata like GPS data, Camera orientation, FOV, ... for each camera and each frame
- Unfortunately, 2x more voluminous than lerobot .mp4 videos with the same data

---

## Streaming: Breaking the Bandwidth Barrier

Image streaming using AV1 provides state of the art bitrate for both rgb and depth data making it possible to reduce bandwidth, reduce latency, and increase fps for robotics. This enables putting robot in outdoor environment where 5G network can only provide limited bandwidth.

For the sake of simplicity, I have compared .AV1 encoding to simpler format like .jpeg for rgb and .tiff for depth (as jpeg does not allow higher level bit depth) and showcase that AV1 provides significant streaming gains from encoding lossy depth data.

### RGB Image

#### File Size Reduction: 36% Smaller

| Resolution | JPEG | AV1 | Savings |
|------------|------|-----|---------|
| 720p | 0.5 MB | 0.32 MB | 36% |
| 480p | 0.32 MB | 0.22 MB | 31% |

#### Frame Rate Boost @ 10 Mbps

| Resolution | JPEG | AV1 | Improvement |
|------------|------|-----|-------------|
| 720p | 20 fps | 32 fps | +60% |
| 480p | 32 fps | 45 fps | +41% |

#### Latency Reduction @ 20fps

| Resolution | JPEG | AV1 | Faster By |
|------------|------|-----|-----------|
| 720p | 50ms | 32ms | 36% |
| 480p | 32ms | 22ms | 31% |

### Depth Image (12bit)

| Format            | 720p      | 480p     |
| ----------------- | --------- | -------- |
| AV1 (12bit lossy) | **13 KB** | **6 KB** |
| TIFF (lossless)   | 266 KB    | 85 KB    |
| RAW               | 14 MB     | 4 MB     |

12-bit = 4096mm = 4m depth precision making it more than enough for most depth camera and most robotic manipulation use case.

[AV1](https://en.wikipedia.org/wiki/AV1) is the only format that delivers on monochrome 12bit as the previous generation [VP9](https://en.wikipedia.org/wiki/VP9) was not able to do so.

### RGB + Depth Combined

| Resolution | Metric | AV1 | JPEG | AV1 Advantage |
|------------|--------|-----|------|---------------|
| **720p** | Size | 333 KB | 750 KB | 60% smaller |
| **720p** | Max FPS | 25 fps | 12 fps | 2.1x faster |
| **720p** | Data Transfer Latency | 42ms | 250ms | 83% lower |
| **480p** | Size | 226 KB | 400 KB | 45% smaller |
| **480p** | Max FPS | 38 fps | 18 fps | 1.9x faster |
| **480p** | Data Transfer Latency | 26ms | 100ms | 74% lower |

#### End-to-End Latency (with encoding/decoding dependent on hardware)

| Resolution | AV1 Total | JPEG Total | Improvement |
|------------|-----------|------------|-------------|
| 720p | 250ms | 525ms | 2.1x faster |
| 480p | 120ms | 225ms | 1.9x faster |

**Code:**

- 12bit monochrome lossy encoding for depth for rust in [avif-serialize](https://github.com/kornelski/avif-serialize)
- [dora-rav1e](https://github.com/dora-rs/dora/blob/dcfaee05b7b1c18a754aec967552fdeef280f6b2/node-hub/dora-rav1e)
- [dora-dav1d](https://github.com/dora-rs/dora/blob/dcfaee05b7b1c18a754aec967552fdeef280f6b2/node-hub/dora-dav1d)

---

## Training: 4x Faster Performance

Decoding single image .avif files instead of decoding videos provides speed up in training when compared with state of the art videos decoder like Torchcodec.

And .mp4 videos are actually not optimized for random read which make it slow to retrieve the actual position of the frame within the .mp4 file.

But, if we save each image as single files and then use filesystem to read them, we will have zero penalty for random frame access.

I wrote a rust based parallel random read & decode python extension, called images-rs, to showcase those speedup.

### Batch Size Performance (720p @ 30fps)

| Frames | images-rs | TorchCodec | Speedup |
|--------|-----------|------------|---------|
| 10 | 46ms | 108ms | 2.3x |
| 32 | 100ms | 346ms | 3.5x |
| 64 | 163ms | 652ms | 4.0x |

**Note**: Torchcodec search for the frame in the mp4, images-rs search for images within an image folder containing each frame as a single image.

images-rs = Rust Parallel Image Reader (Higher FPS is better)

**Code:**

- [images-rs](https://github.com/haixuanTao/images-rs)
- [TorchCodec Benchmark](https://github.com/haixuanTao/torchcodec/tree/images_rs)
- [Dataset Used](https://drive.google.com/file/d/1kSr4i0hMMGbPf89tCeSWKambrdQjKzPl/view?usp=sharing)

---

## Storage: The Trade-off

Storing single file images takes more storage than mp4, about 2x more than lerobot mp4 format.

Although, this can also be seen as a feature as it makes it easier to:

- Enable streaming single images instead of having to download the mp4 videos. Mp4 streaming in the likes of TorchCodec need to partially download the mp4 until reaching the random image keyframe, making it a lot more heavy on ingress cost.
- Enable longer length episodes or even 24/7 robotics data collection without depending on a single file to hold all the data.
- Enable **finegrained metadata** encoded in each images so that we can for example store GPS data for each image, GPS orientation, FOV, Focus, lighting, ... I believe that those fine details at the camera and frame level can have a great impact on the performance of the model.

The additional storage cost should be balanced with reduced GPU data loading time and reduced ingress cost when compared to mp4 streaming.

**Code:**

- Adding EXIF metadata into avif file in [avif-serialize](https://github.com/kornelski/avif-serialize) PR: https://github.com/kornelski/avif-serialize/pull/14
- Adding EXIF metadata formatting in AVIF format in [little_exif](https://github.com/TechnikTobi/little_exif) PR: https://github.com/TechnikTobi/little_exif/pull/62

*Benchmarked on rtx5080 with 24cpus. Results vary by implementation.*

---

## Community Comments

**ramkumarkoppu** - Aug 28, 2025:

> Good work Tao. 1) What's the CPU time for AV1 if GPU is not being used? 2) Did you find ways to reduce the storage for AV1 when it streamed and loaded into the device which has constrained storage and memory?

**haixuantao** (Author) - Aug 29, 2025:

> GPU is never used. Decoding is actually very fast. I mean for streaming you might not need store it? But things you can do to improve storage is: batch frame within 1 avif file, and tweak avif settings depending on what you need.
# AV1 for robotics AI streaming, training and storage

**Published:** August 27, 2025
**Author:** Haixuan Tao
**Type:** Community Article

**Source:** https://huggingface.co/blog/haixuantao/av1-for-ai-and-robotics

---

## tldr: AV1 Performance Gains

**Streaming:** Streaming AV1 packet for rgb and depth is better than alternatives like .jpeg or .tiff

- 60% smaller files/bitrate for rgb+depth (0.32MB vs 0.75MB @ 720p) compared to closest alternatives
- 2x higher frame rates (25fps vs 12fps @ 720p)
- 2x lower latency for RGB+Depth (250ms vs 525ms @ 720p)

**Training:** Batch reading .avif files instead of loading .mp4 videos and retrieving random frames is way faster

- 4x faster than TorchCodec using single .avif image with a parallel image reader instead of videos
- Reading single .avif frame is a lot more simple than mp4 as it removes ffmpeg dependency
- Enable just-in-time dataset image downloading allowing to not wait videos download to start training

**Storage:** Replacing .mp4 videos with .avif single file add more granularity at the cost of more storage

- Enable more variety of dataset like continuous robotic data or streamed data that does not have clear start and end of episodes
- Enable frame metadata like GPS data, Camera orientation, FOV, ... for each camera and each frame
- Unfortunately, 2x more voluminous than lerobot .mp4 videos with the same data

---

## Streaming: Breaking the Bandwidth Barrier

Image streaming using AV1 provides state of the art bitrate for both rgb and depth data making it possible to reduce bandwidth, reduce latency, and increase fps for robotics. This enables putting robot in outdoor environment where 5G network can only provide limited bandwidth.

For the sake of simplicity, I have compared .AV1 encoding to simpler format like .jpeg for rgb and .tiff for depth (as jpeg does not allow higher level bit depth) and showcase that AV1 provides significant streaming gains from encoding lossy depth data.

### RGB Image

#### File Size Reduction: 36% Smaller

| Resolution | JPEG | AV1 | Savings |
|------------|------|-----|---------|
| 720p | 0.5 MB | 0.32 MB | 36% |
| 480p | 0.32 MB | 0.22 MB | 31% |

#### Frame Rate Boost @ 10 Mbps

| Resolution | JPEG | AV1 | Improvement |
|------------|------|-----|-------------|
| 720p | 20 fps | 32 fps | +60% |
| 480p | 32 fps | 45 fps | +41% |

#### Latency Reduction @ 20fps

| Resolution | JPEG | AV1 | Faster By |
|------------|------|-----|-----------|
| 720p | 50ms | 32ms | 36% |
| 480p | 32ms | 22ms | 31% |

### Depth Image (12bit)

| Format            | 720p      | 480p     |
| ----------------- | --------- | -------- |
| AV1 (12bit lossy) | **13 KB** | **6 KB** |
| TIFF (lossless)   | 266 KB    | 85 KB    |
| RAW               | 14 MB     | 4 MB     |

12-bit = 4096mm = 4m depth precision making it more than enough for most depth camera and most robotic manipulation use case.

[AV1](https://en.wikipedia.org/wiki/AV1) is the only format that delivers on monochrome 12bit as the previous generation [VP9](https://en.wikipedia.org/wiki/VP9) was not able to do so.

### RGB + Depth Combined

| Resolution | Metric | AV1 | JPEG | AV1 Advantage |
|------------|--------|-----|------|---------------|
| **720p** | Size | 333 KB | 750 KB | 60% smaller |
| **720p** | Max FPS | 25 fps | 12 fps | 2.1x faster |
| **720p** | Data Transfer Latency | 42ms | 250ms | 83% lower |
| **480p** | Size | 226 KB | 400 KB | 45% smaller |
| **480p** | Max FPS | 38 fps | 18 fps | 1.9x faster |
| **480p** | Data Transfer Latency | 26ms | 100ms | 74% lower |

#### End-to-End Latency (with encoding/decoding dependent on hardware)

| Resolution | AV1 Total | JPEG Total | Improvement |
|------------|-----------|------------|-------------|
| 720p | 250ms | 525ms | 2.1x faster |
| 480p | 120ms | 225ms | 1.9x faster |

**Code:**

- 12bit monochrome lossy encoding for depth for rust in [avif-serialize](https://github.com/kornelski/avif-serialize)
- [dora-rav1e](https://github.com/dora-rs/dora/blob/dcfaee05b7b1c18a754aec967552fdeef280f6b2/node-hub/dora-rav1e)
- [dora-dav1d](https://github.com/dora-rs/dora/blob/dcfaee05b7b1c18a754aec967552fdeef280f6b2/node-hub/dora-dav1d)

---

## Training: 4x Faster Performance

Decoding single image .avif files instead of decoding videos provides speed up in training when compared with state of the art videos decoder like Torchcodec.

And .mp4 videos are actually not optimized for random read which make it slow to retrieve the actual position of the frame within the .mp4 file.

But, if we save each image as single files and then use filesystem to read them, we will have zero penalty for random frame access.

I wrote a rust based parallel random read & decode python extension, called images-rs, to showcase those speedup.

### Batch Size Performance (720p @ 30fps)

| Frames | images-rs | TorchCodec | Speedup |
|--------|-----------|------------|---------|
| 10 | 46ms | 108ms | 2.3x |
| 32 | 100ms | 346ms | 3.5x |
| 64 | 163ms | 652ms | 4.0x |

**Note**: Torchcodec search for the frame in the mp4, images-rs search for images within an image folder containing each frame as a single image.

images-rs = Rust Parallel Image Reader (Higher FPS is better)

**Code:**

- [images-rs](https://github.com/haixuanTao/images-rs)
- [TorchCodec Benchmark](https://github.com/haixuanTao/torchcodec/tree/images_rs)
- [Dataset Used](https://drive.google.com/file/d/1kSr4i0hMMGbPf89tCeSWKambrdQjKzPl/view?usp=sharing)

---

## Storage: The Trade-off

Storing single file images takes more storage than mp4, about 2x more than lerobot mp4 format.

Although, this can also be seen as a feature as it makes it easier to:

- Enable streaming single images instead of having to download the mp4 videos. Mp4 streaming in the likes of TorchCodec need to partially download the mp4 until reaching the random image keyframe, making it a lot more heavy on ingress cost.
- Enable longer length episodes or even 24/7 robotics data collection without depending on a single file to hold all the data.
- Enable **finegrained metadata** encoded in each images so that we can for example store GPS data for each image, GPS orientation, FOV, Focus, lighting, ... I believe that those fine details at the camera and frame level can have a great impact on the performance of the model.

The additional storage cost should be balanced with reduced GPU data loading time and reduced ingress cost when compared to mp4 streaming.

**Code:**

- Adding EXIF metadata into avif file in [avif-serialize](https://github.com/kornelski/avif-serialize) PR: https://github.com/kornelski/avif-serialize/pull/14
- Adding EXIF metadata formatting in AVIF format in [little_exif](https://github.com/TechnikTobi/little_exif) PR: https://github.com/TechnikTobi/little_exif/pull/62

*Benchmarked on rtx5080 with 24cpus. Results vary by implementation.*

---

## Community Comments

**ramkumarkoppu** - Aug 28, 2025:

> Good work Tao. 1) What's the CPU time for AV1 if GPU is not being used? 2) Did you find ways to reduce the storage for AV1 when it streamed and loaded into the device which has constrained storage and memory?

**haixuantao** (Author) - Aug 29, 2025:

> GPU is never used. Decoding is actually very fast. I mean for streaming you might not need store it? But things you can do to improve storage is: batch frame within 1 avif file, and tweak avif settings depending on what you need.
