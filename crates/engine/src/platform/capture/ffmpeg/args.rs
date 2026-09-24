//! The command line of a capture and what ffmpeg said back: the
//! platform's input for a device, the raw output, the modes a camera
//! lists when it refuses the asked ones, and the state its stderr means.
//! Pure, tested without a process.

use super::super::{CaptureError, CaptureSettings, CaptureState, Device, DeviceKind};

/// The platform's capture input, as ffmpeg names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// macOS: `-f avfoundation -i "<video>:<audio>"`, by index or name.
    AvFoundation,
    /// windows: `-f dshow -i video=<name>` / `audio=<name>`, by name only.
    DirectShow,
    /// linux: `-f v4l2 -i /dev/video<n>` and `-f alsa -i default|hw:<n>`.
    Video4Linux,
}

impl Input {
    /// The input of the target this was built for.
    pub const HERE: Input = if cfg!(target_os = "macos") {
        Input::AvFoundation
    } else if cfg!(windows) {
        Input::DirectShow
    } else {
        Input::Video4Linux
    };
}

/// A mode a camera listed: its size and the range of rates it takes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Mode {
    pub width: u32,
    pub height: u32,
    pub min: f32,
    pub max: f32,
}

/// A mode picked to start with: the size and the rate to ask for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Picked {
    pub width: u32,
    pub height: u32,
    pub rate: f32,
}

/// The arguments before `-i` and `-i` itself: the platform's input for
/// `device`, with a mode the camera listed when there is one.
pub(crate) fn input_args(
    input: Input,
    device: &Device,
    mode: Option<Picked>,
) -> Result<Vec<String>, CaptureError> {
    let mut args: Vec<String> = Vec::new();
    let named = !device.name.is_empty();
    let format = match (input, device.kind) {
        (Input::AvFoundation, _) => "avfoundation",
        (Input::DirectShow, _) => "dshow",
        (Input::Video4Linux, DeviceKind::Camera) => "v4l2",
        (Input::Video4Linux, DeviceKind::Microphone) => "alsa",
    };
    args.extend(["-f".to_owned(), format.to_owned()]);
    if let (Some(mode), DeviceKind::Camera) = (mode, device.kind) {
        args.extend([
            "-framerate".to_owned(),
            rate_arg(mode.rate),
            "-video_size".to_owned(),
            format!("{}x{}", mode.width, mode.height),
        ]);
    }
    let which = if named {
        device.name.clone()
    } else {
        device.index.to_string()
    };
    let source = match (input, device.kind) {
        (Input::AvFoundation, DeviceKind::Camera) => format!("{which}:none"),
        (Input::AvFoundation, DeviceKind::Microphone) => format!("none:{which}"),
        (Input::DirectShow, kind) if named => match kind {
            DeviceKind::Camera => format!("video={which}"),
            DeviceKind::Microphone => format!("audio={which}"),
        },
        (Input::DirectShow, kind) => {
            return Err(CaptureError::NoDevice(format!(
                "dshow opens a {} by its name, and none was given",
                kind.noun()
            )));
        }
        (Input::Video4Linux, DeviceKind::Camera) if named => which,
        (Input::Video4Linux, DeviceKind::Camera) => format!("/dev/video{which}"),
        (Input::Video4Linux, DeviceKind::Microphone) if named => which,
        (Input::Video4Linux, DeviceKind::Microphone) if device.index == 0 => "default".to_owned(),
        (Input::Video4Linux, DeviceKind::Microphone) => format!("hw:{which}"),
    };
    args.extend(["-i".to_owned(), source]);
    Ok(args)
}

/// The arguments after the input: the camera's frames filled into the
/// asked size (scaled to cover it and cropped at the center, so any
/// device's aspect gives the same size) at the asked rate, raw RGBA; the
/// microphone's samples mono f32 at the asked rate. Both to stdout.
pub(crate) fn output_args(kind: DeviceKind, settings: &CaptureSettings) -> Vec<String> {
    let (w, h) = (settings.width, settings.height);
    let mut args: Vec<String> = match kind {
        DeviceKind::Camera => vec![
            "-vf".to_owned(),
            format!(
                "scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h},fps={}",
                settings.fps
            ),
            "-f".to_owned(),
            "rawvideo".to_owned(),
            "-pix_fmt".to_owned(),
            "rgba".to_owned(),
        ],
        DeviceKind::Microphone => vec![
            "-f".to_owned(),
            "f32le".to_owned(),
            "-ac".to_owned(),
            "1".to_owned(),
            "-ar".to_owned(),
            settings.rate.to_string(),
        ],
    };
    args.push("pipe:1".to_owned());
    args
}

/// The whole command line after the program.
pub(crate) fn args(
    input: Input,
    kind: DeviceKind,
    settings: &CaptureSettings,
    mode: Option<Picked>,
) -> Result<Vec<String>, CaptureError> {
    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-loglevel", "error"]
        .map(str::to_owned)
        .to_vec();
    args.extend(input_args(input, settings.device(kind), mode)?);
    args.extend(output_args(kind, settings));
    Ok(args)
}

/// A rate as `-framerate` takes it: whole, or to the thousandth.
pub(crate) fn rate_arg(rate: f32) -> String {
    if rate.fract() == 0.0 {
        format!("{}", rate as u32)
    } else {
        format!("{rate:.3}")
    }
}

/// The modes a camera listed when it refused the asked ones: every
/// `<w>x<h>@[<min> <max>]fps` in the lines, whatever prefix ffmpeg put
/// before it.
pub(crate) fn parse_modes(lines: &[String]) -> Vec<Mode> {
    lines
        .iter()
        .filter_map(|line| {
            let (before, after) = line.split_once("@[")?;
            let size = before.split_whitespace().last()?;
            let (w, h) = size.split_once('x')?;
            let (range, _) = after.split_once("]fps")?;
            let (min, max) = range.split_once(' ')?;
            Some(Mode {
                width: w.parse().ok()?,
                height: h.parse().ok()?,
                min: min.trim().parse().ok()?,
                max: max.trim().parse().ok()?,
            })
        })
        .collect()
}

/// The mode to start again with, when the camera refused its default and
/// listed what it takes: one at least the asked size and taking the
/// asked rate if there is one, else the closest, the smallest such; the
/// asked rate if the mode takes it, else the nearest end of its range.
pub(crate) fn pick_mode(modes: &[Mode], settings: &CaptureSettings) -> Option<Picked> {
    let fps = settings.fps as f32;
    let covers = |m: &Mode| m.width >= settings.width && m.height >= settings.height;
    let takes = |m: &Mode| m.min <= fps && fps <= m.max;
    let best = modes.iter().min_by_key(|m| {
        (
            !covers(m),
            !takes(m),
            u64::from(m.width) * u64::from(m.height),
        )
    })?;
    Some(Picked {
        width: best.width,
        height: best.height,
        rate: fps.clamp(best.min, best.max),
    })
}

/// A camera refused the rate or the size it was opened with and listed
/// its modes.
pub(crate) fn refused_mode(lines: &[String]) -> bool {
    lines
        .iter()
        .any(|l| l.contains("is not supported by the device"))
}

/// What a child that ended said, as a state: the system's refusal
/// (`Denied`), a device that is not there (`NoDevice`), or its most
/// telling line (`Failed`). Every line is read: ffmpeg ends with a
/// generic "Error opening input" or "Input/output error" after the line
/// that says why.
pub(crate) fn classify(lines: &[String]) -> CaptureState {
    const DENIED: [&str; 5] = [
        "not authorized",
        "Not authorized",
        "Permission denied",
        "Operation not permitted",
        "Cannot use",
    ];
    const NO_DEVICE: [&str; 8] = [
        "Invalid device index",
        "Video device not found",
        "Audio device not found",
        "No AV capture device found",
        "Could not find video device",
        "Could not find audio",
        "No such device",
        "No such file or directory",
    ];
    if lines.iter().any(|l| DENIED.iter().any(|m| l.contains(m))) {
        return CaptureState::Denied;
    }
    if lines
        .iter()
        .any(|l| NO_DEVICE.iter().any(|m| l.contains(m)))
    {
        return CaptureState::NoDevice;
    }
    CaptureState::Failed(telling(lines))
}

/// The last line that says why, past ffmpeg's generic closing lines.
fn telling(lines: &[String]) -> String {
    let said = |l: &&String| !l.trim().is_empty();
    let generic = |l: &&String| {
        l.contains("Error opening input")
            || l.trim_end().ends_with("Input/output error")
            || l.contains("Exiting normally")
    };
    lines
        .iter()
        .rev()
        .filter(said)
        .find(|l| !generic(l))
        .or_else(|| lines.iter().rev().find(said))
        .map(|l| l.trim().to_owned())
        .unwrap_or_else(|| "ffmpeg ended without a word".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_owned).collect()
    }

    #[test]
    fn each_platform_opens_its_input_by_index_or_name() {
        let cam = Device::first(DeviceKind::Camera);
        let mic = Device::first(DeviceKind::Microphone);
        let join = |a: Result<Vec<String>, CaptureError>| a.unwrap().join(" ");
        assert_eq!(
            join(input_args(Input::AvFoundation, &cam, None)),
            "-f avfoundation -i 0:none"
        );
        assert_eq!(
            join(input_args(Input::AvFoundation, &mic, None)),
            "-f avfoundation -i none:0"
        );
        assert_eq!(
            join(input_args(
                Input::AvFoundation,
                &Device::named(DeviceKind::Camera, "FaceTime HD Camera"),
                None
            )),
            "-f avfoundation -i FaceTime HD Camera:none"
        );
        assert_eq!(
            join(input_args(
                Input::Video4Linux,
                &Device::at(DeviceKind::Camera, 2),
                None
            )),
            "-f v4l2 -i /dev/video2"
        );
        assert_eq!(
            join(input_args(Input::Video4Linux, &mic, None)),
            "-f alsa -i default"
        );
        assert_eq!(
            join(input_args(
                Input::Video4Linux,
                &Device::at(DeviceKind::Microphone, 1),
                None
            )),
            "-f alsa -i hw:1"
        );
        assert_eq!(
            join(input_args(
                Input::DirectShow,
                &Device::named(DeviceKind::Microphone, "Microphone (USB)"),
                None
            )),
            "-f dshow -i audio=Microphone (USB)"
        );
        assert_eq!(
            input_args(Input::DirectShow, &cam, None),
            Err(CaptureError::NoDevice(
                "dshow opens a camera by its name, and none was given".to_owned()
            ))
        );
        // a mode the camera listed goes before the input; never for sound
        let mode = Some(Picked {
            width: 1280,
            height: 720,
            rate: 30.0,
        });
        assert_eq!(
            join(input_args(Input::AvFoundation, &cam, mode)),
            "-f avfoundation -framerate 30 -video_size 1280x720 -i 0:none"
        );
        assert_eq!(
            join(input_args(Input::AvFoundation, &mic, mode)),
            "-f avfoundation -i none:0"
        );
    }

    #[test]
    fn the_output_is_raw_rgba_of_the_asked_size_or_raw_mono_f32() {
        let s = CaptureSettings::default();
        assert_eq!(
            args(Input::AvFoundation, DeviceKind::Camera, &s, None)
                .unwrap()
                .join(" "),
            "-hide_banner -nostdin -loglevel error -f avfoundation -i 0:none \
             -vf scale=640:360:force_original_aspect_ratio=increase,crop=640:360,fps=15 \
             -f rawvideo -pix_fmt rgba pipe:1"
        );
        assert_eq!(
            args(Input::AvFoundation, DeviceKind::Microphone, &s, None)
                .unwrap()
                .join(" "),
            "-hide_banner -nostdin -loglevel error -f avfoundation -i none:0 \
             -f f32le -ac 1 -ar 16000 pipe:1"
        );
        assert_eq!(rate_arg(29.97), "29.970");
        assert_eq!(rate_arg(15.0), "15");
    }

    /// What avfoundation prints when a camera does not take the default
    /// rate, as its format strings in libavdevice have it.
    const REFUSED: &str = "\
[avfoundation @ 0x13b004c80] Selected framerate (29.970030) is not supported by the device.
[avfoundation @ 0x13b004c80] Supported modes:
[avfoundation @ 0x13b004c80]   1920x1080@[1.000000 30.000000]fps
[avfoundation @ 0x13b004c80]   1280x720@[30.000000 30.000000]fps
[avfoundation @ 0x13b004c80]   640x480@[15.000000 30.000000]fps
[in#0 @ 0x600003a0c000] Error opening input: Input/output error
Error opening input file 0:none.
Error opening input files: Input/output error";

    #[test]
    fn a_camera_that_refuses_its_default_is_started_again_with_a_mode_it_listed() {
        let said = lines(REFUSED);
        assert!(refused_mode(&said));
        let modes = parse_modes(&said);
        assert_eq!(modes.len(), 3);
        assert_eq!(
            modes[2],
            Mode {
                width: 640,
                height: 480,
                min: 15.0,
                max: 30.0
            }
        );
        let s = CaptureSettings::default();
        assert_eq!(
            pick_mode(&modes, &s),
            Some(Picked {
                width: 640,
                height: 480,
                rate: 15.0
            }),
            "the smallest that covers 640x360 and takes 15 fps"
        );
        let only_fast = [Mode {
            width: 1280,
            height: 720,
            min: 30.0,
            max: 30.0,
        }];
        assert_eq!(pick_mode(&only_fast, &s).map(|p| p.rate), Some(30.0));
        let too_small = [Mode {
            width: 320,
            height: 240,
            min: 1.0,
            max: 60.0,
        }];
        assert_eq!(pick_mode(&too_small, &s).map(|p| p.width), Some(320));
        assert_eq!(pick_mode(&[], &s), None);
        assert!(!refused_mode(&lines("Invalid device index")));
        // and, had it not been retried, a failure in its own words
        assert_eq!(
            classify(&said),
            CaptureState::Failed(
                "[avfoundation @ 0x13b004c80]   640x480@[15.000000 30.000000]fps".to_owned()
            )
        );
    }

    #[test]
    fn what_ffmpeg_said_is_read_as_a_state_from_every_line() {
        let denied = lines(
            "[avfoundation @ 0x1] Failed to create AV capture input device: Cannot use FaceTime HD Camera\n\
             Error opening input file 0:none.\n\
             Error opening input files: Input/output error",
        );
        assert_eq!(classify(&denied), CaptureState::Denied);
        assert_eq!(
            classify(&lines("/dev/video0: Permission denied")),
            CaptureState::Denied
        );
        for none in [
            "[avfoundation @ 0x1] Invalid device index\nError opening input files: Input/output error",
            "[avfoundation @ 0x1] Video device not found",
            "[avfoundation @ 0x1] Audio device not found",
            "[video4linux2,v4l2 @ 0x1] Cannot open video device /dev/video3: No such file or directory",
            "[dshow @ 0x1] Could not find video device with name [Front] among source devices of type video.",
        ] {
            assert_eq!(classify(&lines(none)), CaptureState::NoDevice, "{none}");
        }
        assert_eq!(
            classify(&lines(
                "[avfoundation @ 0x1] can't add video input to capture session\n\
                 Error opening input files: Input/output error"
            )),
            CaptureState::Failed(
                "[avfoundation @ 0x1] can't add video input to capture session".to_owned()
            )
        );
        assert_eq!(
            classify(&lines("Error opening input files: Input/output error")),
            CaptureState::Failed("Error opening input files: Input/output error".to_owned())
        );
        assert_eq!(
            classify(&[]),
            CaptureState::Failed("ffmpeg ended without a word".to_owned())
        );
    }
}
