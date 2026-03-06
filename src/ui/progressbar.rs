use multi_progressbar::{ProgressBar, TaskProgress};
use supercli::prelude::owo::OwoColorize;

pub struct BrailleProgressBar<T: TaskProgress> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: TaskProgress> BrailleProgressBar<T> {
    pub fn new() -> Self {
        BrailleProgressBar {
            _phantom: std::marker::PhantomData,
        }
    }

    fn block_char(width: usize, progress: f64) -> String {
        let blocks = [' ', '⣀', '⣤', '⣶', '⣷', '⣿'];

        let progress_in_blocks = progress * width as f64;
        let complete = progress_in_blocks.floor() as usize;
        let remainder = progress_in_blocks - complete as f64;

        let mut bar = String::new();
        bar.push_str(&"⣿".bright_blue().to_string().repeat(complete));

        if complete < width {
            let idx = (remainder * (blocks.len() as f64 - 1.0)).floor() as usize;
            bar.push_str(&blocks[idx].bright_blue().to_string());
            bar.push_str(&" ".repeat(width - complete - 1));
        }

        bar
    }
}

pub fn visual_len(s: &str) -> usize {
    let mut len = 0;
    let mut chars = s.chars();

    while let Some(char) = chars.next() {
        if char == '\x1b' {
            if chars.as_str().starts_with('[') {
                chars.next();

                while let Some(char) = chars.next() {
                    if char == 'm' {
                        break;
                    }
                }
            }
        } else {
            len += 1;
        }
    }

    len
}

impl<T: TaskProgress> ProgressBar for BrailleProgressBar<T> {
    type Task = T;

    fn format_line(&self, progress: &Self::Task, width: usize) -> String {
        let (before, after) = (progress.before(), progress.after());
        let (current, total) = progress.progress();

        if total == 0 {
            return " ".repeat(width);
        }

        let before_len = before.as_ref().map(|s| visual_len(s)).unwrap_or(0);
        let after_len = after.as_ref().map(|s| visual_len(s)).unwrap_or(0);

        if before_len + after_len + 2 > width {
            return " ".repeat(width);
        }

        let bar_width = width - before_len - after_len - 2;
        let bar_progress = current as f64 / total as f64;

        let mut bar = String::new();
        bar.push_str(&before.unwrap_or_default());
        bar.push('[');
        bar.push_str(&Self::block_char(bar_width, bar_progress));
        bar.push(']');
        bar.push_str(&after.unwrap_or_default());
        bar
    }
}
