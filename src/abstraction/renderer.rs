use crate::inject::INJECT;
use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::{
    AddScriptToEvaluateOnNewDocumentParams, CaptureScreenshotFormat,
};
use chromiumoxide::page::ScreenshotParams;
use derive_builder::Builder;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

#[derive(Builder)]
#[builder(pattern = "owned", build_fn(error = "anyhow::Error"))]
pub struct WebRenderer {
    fps: u32,
    #[builder(setter(custom))]
    width: u32,
    #[builder(setter(custom))]
    height: u32,
    transparent: bool,
    #[builder(setter(custom))]
    zoom: Option<f64>,
    #[builder(setter(custom))]
    zoom_element: String,
}

impl WebRenderer {
    pub fn builder() -> WebRendererBuilder {
        WebRendererBuilder::create_empty()
    }

    pub async fn render(
        &self,
        input: &Path,
        output: &Path,
        duration: f64,
    ) -> Result<(PathBuf, TempDir)> {
        let total_frames = (duration * self.fps as f64).ceil() as u32;

        eprintln!(
            "Rendering {} frames at {}fps ({:.1}s) — {} -> {}{}",
            total_frames,
            self.fps,
            duration,
            input.display(),
            output.display(),
            if self.transparent {
                " [transparent]"
            } else {
                ""
            },
        );

        let temp_dir = tempfile::tempdir().context("Failed to create temp dir")?;
        let frames_dir = temp_dir.path().to_path_buf();

        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                .window_size(self.width, self.height)
                .build()
                .map_err(|error| anyhow::anyhow!("Browser config error: {error}"))?,
        )
        .await
        .context("Failed to launch Chrome — is Chromium/Chrome installed?")?;

        let _handler = tokio::task::spawn(async move { while handler.next().await.is_some() {} });

        let page = browser.new_page("about:blank").await?;

        page.execute(
            SetDeviceMetricsOverrideParams::builder()
                .width(self.width)
                .height(self.height)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .map_err(|e| anyhow::anyhow!("Viewport config error: {e}"))?,
        )
        .await?;

        page.execute(AddScriptToEvaluateOnNewDocumentParams::new(INJECT))
            .await?;

        let abs_path = input
            .canonicalize()
            .context("Failed to resolve input path")?;

        let url = format!("file://{}", abs_path.display());

        page.goto(&url).await?;

        tokio::time::sleep(Duration::from_millis(300)).await;

        if let Some(zoom) = self.zoom {
            let selector = &self.zoom_element;
            page.evaluate(format!(
                "document.querySelector('{selector}').style.zoom = '{zoom}';"
            ))
            .await?;
        }

        for frame in 0..total_frames {
            let t_ms = frame as f64 / self.fps as f64 * 1000.0;

            page.evaluate(format!("window.__hrec_tick({t_ms:.3})"))
                .await?;

            let data: Vec<u8> = page
                .screenshot(
                    ScreenshotParams::builder()
                        .format(CaptureScreenshotFormat::Png)
                        .omit_background(self.transparent)
                        .build(),
                )
                .await
                .with_context(|| format!("Failed to capture frame {frame}"))?;

            std::fs::write(frames_dir.join(format!("frame_{:06}.png", frame)), &data)
                .with_context(|| format!("Failed to write frame {frame}"))?;

            if frame % self.fps == 0 || frame == total_frames - 1 {
                eprintln!(
                    "  {}/{} frames ({:.0}%)",
                    frame + 1,
                    total_frames,
                    (frame + 1) as f64 / total_frames as f64 * 100.0
                );
            }
        }

        drop(browser);

        let pattern = frames_dir.join("frame_%06d.png");
        Ok((pattern, temp_dir))
    }
}

impl WebRendererBuilder {
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn zoom(mut self, zoom: Option<f64>, element: impl Into<String>) -> Self {
        self.zoom = Some(zoom);
        self.zoom_element = Some(element.into());
        self
    }
}
