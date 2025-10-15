use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use indicatif::{ProgressBar, ProgressStyle};

/// Progress reporter for download operations
pub struct ProgressReporter {
    progress_bar: ProgressBar,
    total_bytes: Arc<AtomicU64>,
    downloaded_bytes: Arc<AtomicU64>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(total_size: u64) -> Self {
        let progress_bar = ProgressBar::new(total_size);

        // Style the progress bar
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Failed to set progress style")
                .progress_chars("#>-")
        );

        Self {
            progress_bar,
            total_bytes: Arc::new(AtomicU64::new(total_size)),
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a spinner for indeterminate progress
    pub fn new_spinner(message: &str) -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to set spinner style")
        );
        progress_bar.set_message(message.to_string());

        Self {
            progress_bar,
            total_bytes: Arc::new(AtomicU64::new(0)),
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Update progress with additional bytes downloaded
    pub fn add_progress(&self, bytes: u64) {
        let current = self.downloaded_bytes.fetch_add(bytes, Ordering::Relaxed);
        self.progress_bar.set_position(current + bytes);
    }

    /// Set the total size if unknown initially
    pub fn set_total(&self, total: u64) {
        self.total_bytes.store(total, Ordering::Relaxed);
        self.progress_bar.set_length(total);
    }

    /// Set a message for the progress bar
    pub fn set_message(&self, message: &str) {
        self.progress_bar.set_message(message.to_string());
    }

    /// Get current progress percentage
    pub fn get_progress_percent(&self) -> f64 {
        let total = self.total_bytes.load(Ordering::Relaxed);
        let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);

        if total == 0 {
            0.0
        } else {
            (downloaded as f64 / total as f64) * 100.0
        }
    }

    /// Mark the progress as complete
    pub fn finish(&self) {
        self.progress_bar.finish_with_message("Done!");
    }

    /// Mark the progress as finished with a custom message
    pub fn finish_with_message(&self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }

    /// Clear the progress bar
    pub fn clear(&self) {
        self.progress_bar.finish_and_clear();
    }
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        self.progress_bar.finish_and_clear();
    }
}