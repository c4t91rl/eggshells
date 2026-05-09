use crypto_core::{SignedManifest, FileEntry};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub speed_bytes_per_sec: f64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Complete,
    Failed(String),
}

pub struct Downloader {
    client: reqwest::Client,
    download_dir: PathBuf,
}

impl Downloader {
    pub fn new(download_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&download_dir).ok();
        Self {
            client: reqwest::Client::builder()
                .user_agent("KryptoUpdate-Client/0.1.0")
                .build()
                .expect("Failed to create HTTP client"),
            download_dir,
        }
    }

    /// Download a file from the update server
    pub async fn download_file(
        &self,
        server_url: &str,
        file_entry: &FileEntry,
        progress_callback: impl Fn(DownloadProgress) + Send + 'static,
    ) -> anyhow::Result<PathBuf> {
        let url = if file_entry.download_url.starts_with("http") {
            file_entry.download_url.clone()
        } else {
            format!(
                "{}{}",
                server_url.trim_end_matches('/'),
                file_entry.download_url
            )
        };

        let response = self.client.get(&url).send().await?;
        let total_size = response.content_length().unwrap_or(file_entry.size);

        let file_path = self.download_dir.join(&file_entry.path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = tokio::fs::File::create(&file_path).await?;
        let mut downloaded: u64 = 0;
        let start_time = std::time::Instant::now();

        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };

            progress_callback(DownloadProgress {
                file_name: file_entry.path.clone(),
                bytes_downloaded: downloaded,
                total_bytes: total_size,
                percentage: (downloaded as f32 / total_size as f32) * 100.0,
                speed_bytes_per_sec: speed,
                status: DownloadStatus::Downloading,
            });
        }

        file.flush().await?;

        progress_callback(DownloadProgress {
            file_name: file_entry.path.clone(),
            bytes_downloaded: downloaded,
            total_bytes: total_size,
            percentage: 100.0,
            speed_bytes_per_sec: 0.0,
            status: DownloadStatus::Complete,
        });

        Ok(file_path)
    }

    /// Download all files in a manifest
    pub async fn download_manifest(
        &self,
        server_url: &str,
        manifest: &SignedManifest,
        progress_callback: impl Fn(DownloadProgress) + Send + Clone + 'static,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for file_entry in &manifest.manifest.files {
            let cb = progress_callback.clone();
            let path = self.download_file(server_url, file_entry, cb).await?;
            paths.push(path);
        }

        Ok(paths)
    }
}