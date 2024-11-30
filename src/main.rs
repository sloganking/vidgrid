use std::error::Error;
use std::process::Command;

fn create_video_grid(
    vid1_path: &str,
    vid2_path: &str,
    vid3_path: &str,
    vid4_path: &str,
    duration: u32,
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let scale_pad = "scale=iw*min(960/iw\\,540/ih):ih*min(960/iw\\,540/ih),\
                     pad=960:540:(960-iw*min(960/iw\\,540/ih))/2:\
                     (540-ih*min(960/iw\\,540/ih))/2";

    let videos = vec![
        ("0:v", "vid1"),
        ("1:v", "vid2"),
        ("2:v", "vid3"),
        ("3:v", "vid4"),
    ];
    let mut filters = Vec::new();

    for (input, label) in &videos {
        let filter = format!("[{}]{}[{}];", input, scale_pad, label);
        filters.push(filter);
    }

    filters.push("[vid1][vid2]hstack=inputs=2[top];".to_string());
    filters.push("[vid3][vid4]hstack=inputs=2[bottom];".to_string());
    filters.push("[top][bottom]vstack=inputs=2[final]".to_string());

    let filter_complex = filters.join(" ");

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
        .arg(output_path)
        .status()?;

    if !status.success() {
        return Err("ffmpeg command failed".into());
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    create_video_grid(
        "c:\\Users\\Brioche Elm\\Videos\\crafting\\video_maker\\rotating_square.mp4",
        "c:\\Users\\Brioche Elm\\Videos\\crafting\\video_maker\\rainbow.mp4",
        "c:\\Users\\Brioche Elm\\Videos\\crafting\\video_maker\\mandelbrot.mp4",
        "c:\\Users\\Brioche Elm\\Videos\\crafting\\video_maker\\grid.mp4",
        50,
        "output.mp4",
    )?;
    Ok(())
}
