use crate::abstraction::{Ffmpeg, SupportedExts, WebRenderer};
use crate::commands::prelude::*;

#[async_trait]
impl CliSubcommand for Render {
    async fn run(self: Box<Self>) -> Result<()> {
        if self.transparent
            && !matches!(
                self.output
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("webm" | "mov")
            )
        {
            styled_bail!(
                "`--transparent` requires a `{}` or `{}` output file",
                (".webm", "command"),
                (".mov", "command")
            );
        }

        let renderer = WebRenderer::builder()
            .fps(self.fps)
            .dimensions(self.width, self.height)
            .transparent(self.transparent)
            .zoom(self.zoom, self.zoom_element)
            .build()?;

        let (path_pattern, _frames_guard) = renderer
            .render(&self.input, &self.output, self.duration)
            .await?;

        let args = SupportedExts::infer(&self.output)?.ffmpeg_args();

        Ffmpeg::builder()
            .input(path_pattern)
            .fps(self.fps)
            .output(self.output)
            .args(&args)
            .run()?;

        Ok(())
    }
}
