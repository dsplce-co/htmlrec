use crate::patched::multi_progressbar::TaskProgress;

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use unicode_truncate::UnicodeTruncateStr;

pub struct FrameProgress {
    inner: Arc<FrameProgressInner>,
}

struct FrameProgressInner {
    current: AtomicU32,
    total: u32,
    label: String,
}

static MAX_LABEL_LENGTH: usize = 24;

impl FrameProgress {
    pub fn new(total: u32, label: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(FrameProgressInner {
                current: AtomicU32::new(0),
                total,
                label: label.into(),
            }),
        }
    }

    pub fn increment(&self) {
        self.inner.current.fetch_add(1, Ordering::Relaxed);
    }
}

impl Clone for FrameProgress {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl TaskProgress for FrameProgress {
    fn progress(&self) -> (u64, u64) {
        (
            self.inner.current.load(Ordering::Relaxed) as u64,
            self.inner.total as u64,
        )
    }

    fn before(&self) -> Option<String> {
        let current = self.inner.current.load(Ordering::Relaxed);
        let percent = format!("{:.0}%", current as f64 / self.inner.total as f64 * 100.0);
        let aligned = format!("{:>4}", percent);
        let counter = format!("{}/{}", current, self.inner.total);

        let aligned_counter = format!(
            "{:>width$}",
            counter,
            width = self.inner.total.to_string().len() * 2 + 1
        );

        Some(format!(
            " {} {} ",
            styled_msg!("{}", (aligned, "number")),
            styled_msg!("{}", (aligned_counter, "muted")),
        ))
    }

    fn after(&self) -> Option<String> {
        let label = self.inner.label.as_str();

        let aligned = if label.len() > MAX_LABEL_LENGTH - 1 {
            format!("{}…", label.unicode_truncate(MAX_LABEL_LENGTH).0)
        } else {
            label.to_string()
        };

        let aligned = aligned.unicode_pad(
            MAX_LABEL_LENGTH + 1,
            unicode_truncate::Alignment::Left,
            true,
        );

        Some(styled_msg!(" {}", (aligned.to_string(), "file_path")))
    }
}
