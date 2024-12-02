use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

mod options;

fn get_video_framerate(video_path: &Path) -> Result<f64, Box<dyn Error>> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=r_frame_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            video_path.to_str().unwrap(),
        ])
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
            numerator / denominator
        } else {
            return Err(format!("Invalid frame rate format: {}", fps_str).into());
        }
    } else {
        f64::from_str(&fps_str)?
    };

    Ok(fps)
}

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
    // Get frame rates of all input videos
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

    // Calculate individual video dimensions for the 2x2 grid
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

    // Execute the ffmpeg command with the new parameters
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
    Ok(())
}
