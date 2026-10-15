use std::{fs, path::Path};
use anyhow::Context;
use tempfile::TempDir;

pub fn upload_and_rotate(
    archive_path: &Path,
    remote: &str,
    job_name: &str,
    keep_last_days: u32,
) -> anyhow::Result<()> {
    if !archive_path.is_file() {
        anyhow::bail!("Watchdog alert: Archive file '{}' does not exist!", archive_path.display());
    }

    let metadata = fs::metadata(archive_path)
        .with_context(|| format!("Failed to read file metadata for '{}'", archive_path.display()))?;
    let file_size = metadata.len();
    if file_size == 0 {
        anyhow::bail!("Watchdog alert: Archive file '{}' is empty (0 bytes)!", archive_path.display());
    }
    println!("Watchdog OK: Archive verified ({} bytes).", file_size);

    let destination = format!("{}/{}", remote, job_name);

    println!("Starting cloud upload to '{}'...", destination);
    let upload_status = std::process::Command::new("rclone")
        .arg("copy")
        .arg(archive_path)
        .arg(&destination)
        .status()
        .context("Failed to execute rclone. Is rclone installed and available in PATH?")?;

    if !upload_status.success() {
        anyhow::bail!("rclone copy failed with exit code: {:?}", upload_status.code());
    }
    println!("Upload completed successfully!");

    let min_age = format!("{}d", keep_last_days);
    println!("Rotating old backups (deleting archives older than {})...", min_age);
    let delete_status = std::process::Command::new("rclone")
        .arg("delete")
        .arg(&destination)
        .arg("--min-age")
        .arg(&min_age)
        .status()
        .context("Failed to execute rclone delete")?;

    if !delete_status.success() {
        anyhow::bail!("rclone delete failed with exit code: {:?}", delete_status.code());
    }

    fs::remove_file(archive_path)
        .with_context(|| format!("Failed to clean up local archive file '{}'", archive_path.display()))?;
    println!("Local temporary archive cleaned up successfully.");

    Ok(())
}

pub fn list_cloud_backups(remote: &str, job_name: &str) -> anyhow::Result<Vec<String>> {
    let destination = format!("{}/{}", remote, job_name);
    let output = std::process::Command::new("rclone")
        .arg("lsf")
        .arg(&destination)
        .output()
        .context("Failed to list files from rclone remote")?;

    if !output.status.success() {
        anyhow::bail!("rclone listing failed with exit code: {:?}", output.status.code());
    }

    let text = String::from_utf8(output.stdout)
        .context("Failed to parse rclone output as UTF-8 string")?;
    let mut files: Vec<String> = text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    files.sort();
    Ok(files)
}

pub fn download_cloud_backup(
    remote: &str,
    job_name: &str,
    file_name: &str,
) -> anyhow::Result<(TempDir, std::path::PathBuf)> {
    let temp_dir = tempfile::Builder::new()
        .prefix("pi_backup_restore_")
        .tempdir()?;

    let target_file_path = temp_dir.path().join(file_name);
    let source = format!("{}/{}/{}", remote, job_name, file_name);

    let copy_status = std::process::Command::new("rclone")
        .arg("copyto")
        .arg(&source)
        .arg(&target_file_path)
        .status()
        .context("Failed to execute rclone copyto")?;

    if !copy_status.success() {
        anyhow::bail!("rclone copyto failed with exit code: {:?}", copy_status.code());
    }

    Ok((temp_dir, target_file_path))
}
