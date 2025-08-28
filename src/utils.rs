use anyhow::Context;
use rand::Rng;
use tempfile::NamedTempFile;
use tokio::fs;
use tokio::process::Command;

pub async fn convert_to_avif(input: Vec<u8>, vf: Option<String>) -> anyhow::Result<Vec<u8>> {
    let input_file = NamedTempFile::new().context("failed to create temp input file")?;
    tokio::fs::write(input_file.path(), &input)
        .await
        .context("failed to write input file")?;
    let mut output_path = std::env::temp_dir();
    output_path.push(format!("{}.avif", generate_unique_id()));

    let mut cmd = Command::new("ffmpeg");
    cmd.args(&[
        "-hide_banner",
        "-loglevel",
        "error",
        "-i",
        input_file.path().to_str().unwrap(),
    ]);
    if let Some(filter) = vf {
        cmd.args(&["-vf", &filter]);
    }
    cmd.args(&[
        "-c:v",
        "libaom-av1",
        "-still-picture",
        "1",
        "-crf",
        "30",
        "-b:v",
        "0",
        "-pix_fmt",
        "yuv420p10le",
        output_path.to_str().unwrap(),
    ]);

    // Run FFmpeg
    let status = cmd.status().await.context("failed to run ffmpeg")?;
    if !status.success() {
        return Err(anyhow::anyhow!("ffmpeg failed to convert image"));
    }

    // Read output file into memory
    let output_bytes = fs::read(&output_path)
        .await
        .context("failed to read output file")?;

    tokio::fs::remove_file(input_file.path()).await.ok();
    tokio::fs::remove_file(&output_path).await.ok();

    Ok(output_bytes)
}

pub fn generate_unique_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut rng = rand::rng();
    (0..12)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
