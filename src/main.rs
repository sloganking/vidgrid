// main.rs
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

mod options;

/// Helper function to retrieve the frame rate of a video using ffprobe
fn get_video_framerate(video_path: &Path) -> Result<f64, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("stream=r_frame_rate")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()?;

    if !output.status.success() {
        return Err(format!("ffprobe failed for {}", video_path.display()).into());
    }

    let fps_str = String::from_utf8(output.stdout)?.trim().to_string();

    // Parse the frame rate string, which might be in the form "30000/1001"
    let fps = if fps_str.contains('/') {
        let parts: Vec<&str> = fps_str.split('/').collect();
        if parts.len() == 2 {
            let numerator = f64::from_str(parts[0])?;
            let denominator = f64::from_str(parts[1])?;
            if denominator == 0.0 {
                return Err(
                    format!("Invalid frame rate denominator in {}", video_path.display()).into(),
                );
            }
            numerator / denominator
        } else {
            return Err(format!("Invalid frame rate format: {}", fps_str).into());
        }
    } else {
        f64::from_str(&fps_str)?
    };

    Ok(fps)
}

/// Helper function to retrieve the duration of a video using ffprobe
fn get_video_duration(video_path: &Path) -> Result<u32, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(video_path)
        .output()?;

    if !output.status.success() {
        return Err(format!("ffprobe failed for {}", video_path.display()).into());
    }

    let dur_str = String::from_utf8(output.stdout)?.trim().to_string();

    // Parse the duration string to f64 and then convert to u32 (seconds)
    let dur_f64 = f64::from_str(&dur_str)?;
    let dur_u32 = dur_f64.floor() as u32;

    Ok(dur_u32)
}

/// Creates a 2x2 video grid from four input videos.
///
/// This function takes four input video files, adjusts their frame rates and durations as specified,
/// and combines them into a single output video arranged in a 2x2 grid layout. The output video
/// will have a resolution defined by `output_width` and `output_height`, and its duration will
/// be the lesser of the longest input video or the specified `duration`.
///
/// # Arguments
///
/// * `vid1_path` - Path to the first video (top-left).
/// * `vid2_path` - Path to the second video (top-right).
/// * `vid3_path` - Path to the third video (bottom-left).
/// * `vid4_path` - Path to the fourth video (bottom-right).
/// * `duration` - Maximum duration of the output video in seconds.
/// * `output_width` - Width of the output video.
/// * `output_height` - Height of the output video.
/// * `max_framerate` - Maximum frame rate for the output video.
/// * `output_path` - Path to save the output video.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Ok on success, Err otherwise.
///
/// # Errors
///
/// Returns an error if:
/// - Any of the input video paths are invalid or inaccessible.
/// - `ffprobe` or `ffmpeg` commands fail to execute.
/// - There is an issue with processing the video streams.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use your_crate::create_video_grid;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     create_video_grid(
///         Path::new("video1.mp4"),
///         Path::new("video2.mp4"),
///         Path::new("video3.mp4"),
///         Path::new("video4.mp4"),
///         60,
///         1920,
///         1080,
///         60.0,
///         Path::new("output.mp4"),
///     )?;
///     Ok(())
/// }
/// ```
fn create_video_grid(
    vid1_path: &Path,
    vid2_path: &Path,
    vid3_path: &Path,
    vid4_path: &Path,
    duration: u32,
    output_width: u32,
    output_height: u32,
    max_framerate: f64,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    // Step 1: Retrieve Frame Rates of All Input Videos
    let fps1 = get_video_framerate(vid1_path)?;
    let fps2 = get_video_framerate(vid2_path)?;
    let fps3 = get_video_framerate(vid3_path)?;
    let fps4 = get_video_framerate(vid4_path)?;

    // Determine the maximum frame rate among the inputs
    let mut max_input_fps = fps1.max(fps2).max(fps3).max(fps4);

    // Cap the frame rate at the specified max_framerate
    if max_input_fps > max_framerate {
        max_input_fps = max_framerate;
    }

    // Step 2: Retrieve Durations of All Input Videos
    let dur1 = get_video_duration(vid1_path)?;
    let dur2 = get_video_duration(vid2_path)?;
    let dur3 = get_video_duration(vid3_path)?;
    let dur4 = get_video_duration(vid4_path)?;

    // Determine the maximum duration among the inputs
    let max_input_duration = dur1.max(dur2).max(dur3).max(dur4);

    // Calculate the output duration: min(user_duration, max_input_duration)
    let output_duration = if duration < max_input_duration {
        duration
    } else {
        max_input_duration
    };

    // Step 3: Calculate Individual Video Dimensions for the 2x2 Grid
    let video_width = output_width / 2;
    let video_height = output_height / 2;

    // Construct the scaling and padding filter with the new resolution
    let scale_pad = format!(
        "scale={vw}:{vh}:force_original_aspect_ratio=decrease,pad={vw}:{vh}:(ow-iw)/2:(oh-ih)/2",
        vw = video_width,
        vh = video_height
    );

    let videos = vec![
        ("0:v", "vid1"),
        ("1:v", "vid2"),
        ("2:v", "vid3"),
        ("3:v", "vid4"),
    ];
    let mut filters = Vec::new();

    // Apply scaling, reset PTS, set dynamic frame rate, and add fifo to each video input
    for (input, label) in &videos {
        let filter = format!(
            "[{input}]{scale_pad},setpts=PTS-STARTPTS,fps=fps={fps},fifo[{label}];",
            input = input,
            scale_pad = scale_pad,
            fps = max_input_fps,
            label = label
        );
        filters.push(filter);
    }

    // Stack the videos into a 2x2 grid
    filters.push("[vid1][vid2]hstack=inputs=2[top];".to_string());
    filters.push("[vid3][vid4]hstack=inputs=2[bottom];".to_string());
    filters.push("[top][bottom]vstack=inputs=2[final]".to_string());

    let filter_complex = filters.join(" ");

    // Step 4: Execute the ffmpeg Command with the New Parameters
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(vid1_path)
        .arg("-i")
        .arg(vid2_path)
        .arg("-i")
        .arg(vid3_path)
        .arg("-i")
        .arg(vid4_path)
        .arg("-filter_complex")
        .arg(&filter_complex)
        .arg("-map")
        .arg("[final]")
        .arg("-t")
        .arg(&output_duration.to_string())
        .arg("-vsync")
        .arg("2") // Ensure frame duplication is handled correctly
        .arg("-y") // Overwrite output file if it exists
        .arg(output_path)
        .status()?;

    if !status.success() {
        return Err("ffmpeg command failed".into());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: options::Args = clap::Parser::parse();

    create_video_grid(
        &args.in1,
        &args.in2,
        &args.in3,
        &args.in4,
        args.duration,
        args.width,
        args.height,
        args.max_framerate,
        &args.output_path,
    )?;

    if args.open {
        open::that(&args.output_path)?;
    }

    Ok(())
}
