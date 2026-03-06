use anyhow::{bail, Context, Result};
use derive_builder::Builder;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use strum::EnumString;

#[derive(EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedExts {
    Mp4,
    Webm,
    Mov,
}

impl SupportedExts {
    pub fn infer(output: &Path) -> Result<Self> {
        let ext = output
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("Output file has no extension"))?;

        SupportedExts::from_str(ext)
            .map_err(|_| anyhow::anyhow!("Unsupported output format: .{ext}"))
    }

    pub fn ffmpeg_args(&self) -> Vec<&'static str> {
        match self {
            SupportedExts::Mp4 => vec![
                "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18", "-preset", "fast",
            ],
            SupportedExts::Webm => vec![
                "-c:v",
                "libvpx-vp9",
                "-pix_fmt",
                "yuva420p",
                "-crf",
                "18",
                "-b:v",
                "0",
            ],
            SupportedExts::Mov => vec![
                "-c:v",
                "prores_ks",
                "-profile:v",
                "4444",
                "-pix_fmt",
                "yuva444p10le",
            ],
        }
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "anyhow::Error"))]
pub struct Ffmpeg {
    #[builder(setter(into))]
    input: PathBuf,
    #[builder(setter(into))]
    output: PathBuf,
    fps: u32,
    #[builder(setter(custom))]
    codec_args: Vec<String>,
}

impl Ffmpeg {
    pub fn builder() -> FfmpegBuilder {
        FfmpegBuilder::create_empty()
    }

    pub fn check() -> Result<()> {
        Command::new("ffmpeg")
            .arg("-version")
            .output()
            .context("ffmpeg not found — please install ffmpeg")?;

        Ok(())
    }

    pub fn run(self) -> Result<()> {
        Ffmpeg::check()?;

        eprintln!("Encoding video...");

        let fps_str = self.fps.to_string();

        let mut cmd_args: Vec<String> = vec![
            "-y".into(),
            "-framerate".into(),
            fps_str,
            "-i".into(),
            self.input.to_str().unwrap().to_string(),
        ];

        cmd_args.extend(self.codec_args);
        cmd_args.push(self.output.to_str().unwrap().to_string());

        let status = Command::new("ffmpeg")
            .args(&cmd_args)
            .status()
            .context("Failed to run ffmpeg")?;

        if !status.success() {
            bail!("ffmpeg failed");
        }

        eprintln!("Done! -> {}", self.output.display());

        Ok(())
    }
}

impl FfmpegBuilder {
    pub fn args(mut self, args: &[&str]) -> Self {
        self.codec_args = Some(args.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn run(self) -> Result<()> {
        self.build()?.run()
    }
}
