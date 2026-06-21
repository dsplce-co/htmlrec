use anyhow::Result;
use chromiumoxide::detection::{default_executable, DetectionOptions};
use chromiumoxide::fetcher::{BrowserFetcher, BrowserFetcherOptions, FetcherError};
use std::path::PathBuf;

use crate::sys;

pub(crate) struct Chrome;

impl Chrome {
    /// Locate a Chrome/Chromium executable, preferring one already installed and
    /// otherwise downloading a cached headless Chromium once.
    pub(crate) async fn resolve_executable() -> Result<PathBuf> {
        if let Ok(path) = default_executable(DetectionOptions::default()) {
            return Ok(path);
        }

        let cache_dir = Self::cache_dir()?;

        let options = BrowserFetcherOptions::builder()
            .with_path(cache_dir.clone())
            .build()
            .map_err(|error| match error {
                FetcherError::UnsupportedOs(os, arch) => anyhow::anyhow!(styled_error!(
                    "No Chrome/Chromium found, and automatic download isn't available for {}. Please install one manually",
                    (format!("{os} {arch}"), "muted")
                )),
                other => anyhow::anyhow!(styled_error!(
                    "Couldn't prepare the Chromium download: {}",
                    (other.to_string(), "muted")
                )),
            })?;

        let downloading = !sys::dir_has_contents(&cache_dir);

        if downloading {
            supercli::info!(&styled_msg!(
                "No Chrome found. Downloading headless Chromium, caching to {} …",
                (cache_dir.display().to_string(), "file_path")
            ));
        }

        std::fs::create_dir_all(&cache_dir).map_err(|error| {
            anyhow::anyhow!(styled_error!(
                "Couldn't create the Chromium cache directory {}: {}",
                (cache_dir.display().to_string(), "file_path"),
                (error.to_string(), "muted")
            ))
        })?;

        let installation = BrowserFetcher::new(options)
            .fetch()
            .await
            .map_err(|error| {
                anyhow::anyhow!(styled_error!(
                    "Failed to download Chromium (have you got network access?): {}",
                    (error.to_string(), "muted")
                ))
            })?;

        if downloading {
            supercli::success!(&styled_msg!(
                "Chromium {} ready, cached at {}",
                (format!("r{}", installation.build_info.id), "number"),
                (cache_dir.display().to_string(), "file_path")
            ));
        }

        Ok(installation.executable_path)
    }

    /// Stable, htmlrec-owned cache directory for the downloaded browser
    /// (`~/.cache/htmlrec/chromium` on Linux; the platform equivalent elsewhere).
    fn cache_dir() -> Result<PathBuf> {
        let base = dirs::cache_dir().ok_or_else(|| {
            anyhow::anyhow!(styled_error!(
                "Couldn't determine a cache directory for the Chromium download"
            ))
        })?;

        Ok(base.join("htmlrec").join("chromium"))
    }
}
