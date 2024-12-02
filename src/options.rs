use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[clap(version)]
pub struct Args {
    /// The path to the first video file. This will be the top-left video in the output grid
    #[clap(long, help_heading = "INPUT")]
    pub in1: PathBuf,

    /// The path to the second video file. This will be the top-right video in the output grid
    #[clap(long, help_heading = "INPUT")]
    pub in2: PathBuf,

    /// The path to the third video file. This will be the bottom-left video in the output grid
    #[clap(long, help_heading = "INPUT")]
    pub in3: PathBuf,

    /// The path to the fourth video file. This will be the bottom-right video in the output grid
    #[clap(long, help_heading = "INPUT")]
    pub in4: PathBuf,

    /// The resolution width of the output video file
    #[clap(long, default_value_t = 1920)]
    pub width: u32,

    /// The resolution height of the output video file
    #[clap(long, default_value_t = 1080)]
    pub height: u32,

    /// the maximum length of the output video in seconds. Videos longer than this will be truncated.
    #[clap(long, default_value_t = 15)]
    pub duration: u32,

    /// The path to which to write the output png file
    #[clap(
        long,
        short = 'o',
        default_value = "output.mp4",
        help_heading = "OUTPUT"
    )]
    pub output_path: PathBuf,
}
