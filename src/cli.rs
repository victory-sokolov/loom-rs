use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "vloom")]
#[command(about = "Record Chrome windows for LLM analysis")]
#[command(version)]
pub struct Args {
    /// Window title filter (default: any Chrome window)
    #[arg(short, long)]
    pub target: Option<String>,

    /// Auto-stop recording after N seconds
    #[arg(short, long)]
    pub duration: Option<u32>,

    /// Output file path (default: ~/vloom-recordings/vloom-{timestamp}.mp4)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Frame rate (default: 30)
    #[arg(long, default_value = "30")]
    pub fps: u32,

    /// Target resolution (default: 1280x720)
    #[arg(long, default_value = "1280x720")]
    pub resolution: String,

    /// List available Chrome windows and exit
    #[arg(short, long)]
    pub list: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn resolution_tuple(&self) -> (u32, u32) {
        let parts: Vec<&str> = self.resolution.split('x').collect();
        if parts.len() != 2 {
            return (1280, 720);
        }
        let w = parts[0].parse().unwrap_or(1280);
        let h = parts[1].parse().unwrap_or(720);
        (w, h)
    }
}
