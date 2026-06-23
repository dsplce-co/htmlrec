use crate::abstraction::FrameProgress;
use crate::inject::INJECT;
use crate::ui::BrailleProgressBar;
use anyhow::{Context, Result};
use atty::Stream;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::{
    AddScriptToEvaluateOnNewDocumentParams, CaptureScreenshotFormat,
};
use chromiumoxide::page::ScreenshotParams;
use derive_builder::Builder;
use futures::StreamExt;
use crate::patched::multi_progressbar::MultiProgressBar;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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

        supercli::styled!(
            "Rendering {} frames @{}fps ({}s) — {} -> {}{}",
            (total_frames.to_string(), "number"),
            (self.fps.to_string(), "number"),
            (format!("{:.1}", duration), "number"),
            (input.display().to_string(), "file_path"),
            (output.display().to_string(), "file_path"),
            (
                if self.transparent {
                    " [transparent]".to_string()
                } else {
                    String::new()
                },
                "muted"
            ),
        );

        let label = output
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        let progress = FrameProgress::new(total_frames, label);
        let tasks: Arc<Mutex<Vec<FrameProgress>>> = Arc::new(Mutex::new(vec![progress.clone()]));

        let mp = atty::is(Stream::Stderr).then(|| {
            Arc::new(MultiProgressBar::new(
                BrailleProgressBar::<FrameProgress>::new(),
                Arc::clone(&tasks),
            ))
        });

        let temp_dir = tempfile::tempdir().context(styled_error!("Failed to create temp dir"))?;
        let frames_dir = temp_dir.path().to_path_buf();

        // Chromium's sandbox can't initialize as root or without user namespaces
        // (e.g. inside a container); HREC_NO_SANDBOX lets such environments opt out.
        let no_sandbox = std::env::var("HREC_NO_SANDBOX")
            .map(|value| !value.is_empty() && value != "0")
            .unwrap_or(false);

        let mut browser_config = BrowserConfig::builder().window_size(self.width, self.height);

        if no_sandbox {
            browser_config = browser_config.no_sandbox();
        }

        let (browser, mut handler) = Browser::launch(
            browser_config
                .build()
                .map_err(|error| {
                    anyhow::anyhow!(styled_error!(
                        "Browser config error: {}",
                        (error.to_string(), "muted")
                    ))
                })?,
        )
        .await
        .context(styled_error!(
            "Failed to launch Chrome — is Chromium/Chrome installed?"
        ))?;

        let _handler = tokio::task::spawn(async move { while handler.next().await.is_some() {} });

        let page = browser.new_page("about:blank").await?;

        page.execute(
            SetDeviceMetricsOverrideParams::builder()
                .width(self.width)
                .height(self.height)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .map_err(|e| {
                    anyhow::anyhow!(styled_error!(
                        "Viewport config error: {}",
                        (e.to_string(), "muted")
                    ))
                })?,
        )
        .await?;

        page.execute(AddScriptToEvaluateOnNewDocumentParams::new(INJECT))
            .await?;

        let abs_path = input
            .canonicalize()
            .context(styled_error!("Failed to resolve input path"))?;

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

        let mp_draw = mp.clone();

        let draw_handle = tokio::task::spawn(async move {
            loop {
                if let Some(ref mp) = mp_draw {
                    mp.draw().ok();
                }

                tokio::time::sleep(Duration::from_millis(16)).await;
            }
        });

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
                .with_context(|| {
                    styled_error!("Failed to capture frame {}", (frame.to_string(), "number"))
                })?;

            std::fs::write(frames_dir.join(format!("frame_{:06}.png", frame)), &data)
                .with_context(|| {
                    styled_error!("Failed to write frame {}", (frame.to_string(), "number"))
                })?;

            progress.increment();
        }

        draw_handle.abort();

        if let Some(ref mp) = mp {
            mp.draw().ok();
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
