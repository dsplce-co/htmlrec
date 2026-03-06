use anyhow::Result;
use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use enum_variant_type::EnumVariantType;
use evt_trait_object::Variants;
use std::path::PathBuf;

#[async_trait]
pub trait CliSubcommand: std::fmt::Debug + Send {
    async fn run(self: Box<Self>) -> Result<()>;
}

#[derive(Parser, Debug)]
#[command(
    name = "hrec",
    about = "Render HTML animations to video",
    version,
    styles = supercli::clap::create_minimal_help_styles()
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand, EnumVariantType, Variants)]
#[variants_trait(CliSubcommand)]
pub enum Commands {
    /// Render an HTML file to video
    #[evt(derive(Debug, Args))]
    Render {
        /// Input HTML file
        input: PathBuf,

        /// Output video file
        #[arg(short, long, default_value = "output.webm")]
        output: PathBuf,

        /// Animation duration in seconds
        #[arg(short, long, default_value_t = 5.0)]
        duration: f64,

        /// Frames per second
        #[arg(short, long, default_value_t = 60)]
        fps: u32,

        /// Viewport width
        #[arg(long, default_value_t = 1280)]
        width: u32,

        /// Viewport height
        #[arg(long, default_value_t = 720)]
        height: u32,

        /// Preserve transparency (outputs WebM/VP9 with alpha; output file should be .webm or .mov)
        #[arg(short, long)]
        transparent: bool,

        /// CSS zoom applied to --zoom-element (e.g. 0.5, 1.5)
        #[arg(long)]
        zoom: Option<f64>,

        /// CSS selector of the element to apply --zoom to (default: :root)
        #[arg(long, default_value = ":root")]
        zoom_element: String,
    },
}
