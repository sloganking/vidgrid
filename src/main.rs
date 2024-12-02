use std::error::Error;
use std::path::Path;
use std::process::Command;

mod options;

fn create_video_grid(
    vid1_path: &Path,
    vid2_path: &Path,
    vid3_path: &Path,
    vid4_path: &Path,
    duration: u32,
    output_width: u32,
    output_height: u32,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
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

    // Apply the scaling and padding filter to each video input
    for (input, label) in &videos {
        let filter = format!(
            "[{input}]{scale_pad}[{label}];",
            input = input,
            scale_pad = scale_pad,
            label = label
        );
        filters.push(filter);
    }

    // Stack the videos into a 2x2 grid
    filters.push("[vid1][vid2]hstack=inputs=2[top];".to_string());
    filters.push("[vid3][vid4]hstack=inputs=2[bottom];".to_string());
    filters.push("[top][bottom]vstack=inputs=2[final]".to_string());

    let filter_complex = filters.join(" ");

    // Execute the ffmpeg command with the new resolution parameters
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
        .arg(&duration.to_string())
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
        &args.output_path,
    )?;
    Ok(())
}
