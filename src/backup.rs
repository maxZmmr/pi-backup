use crate::config::{Config, JobConfig};
use crate::upload::upload_and_rotate;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use anyhow::Context;
use chrono::Local;
use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};
use tempfile::NamedTempFile;

pub fn run_backup(job_name: &str, job: &JobConfig, config: &Config) -> anyhow::Result<()> {
    println!("Job '{}' has {} configured path(s): {:#?}", job_name, job.include_paths.len(), job.include_paths);

    let mut sqlite_temp_file = None;
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let archive_path = PathBuf::from(format!("{}_{}_backup.tar.zst.age", job_name, timestamp));

    if let Some(db_path) = &job.sqlite_db {
        println!("SQLite database configured: {}", db_path.display());
        let snap = create_sqlite_snapshot(db_path)?;
        sqlite_temp_file = Some(snap);
    } else {
        println!("No SQLite database configured for this job.");
    }

    create_backup_archive(
        &archive_path,
        &job.include_paths,
        sqlite_temp_file.as_ref(),
        &config.age_public_key,
    )?;

    upload_and_rotate(
        &archive_path,
        &config.rclone_remote,
        job_name,
        job.keep_last,
    )?;

    Ok(())
}

fn create_sqlite_snapshot(db_path: &Path) -> anyhow::Result<NamedTempFile> {
    println!("Creating consistent online SQLite snapshot from: {}", db_path.display());

    // Prefer in-memory /dev/shm tmpfs on Linux (Raspberry Pi RAM-disk) to avoid SD-card wear
    let temp_file = if Path::new("/dev/shm").exists() {
        tempfile::Builder::new()
            .prefix("backup_sqlite_")
            .tempfile_in("/dev/shm")?
    } else {
        tempfile::Builder::new()
            .prefix("backup_sqlite_")
            .tempfile()?
    };

    let src = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("Failed to open source SQLite database at '{}'", db_path.display()))?;
    let mut dst = Connection::open(temp_file.path())?;
    let backup = Backup::new(&src, &mut dst)?;

    backup.run_to_completion(100, Duration::from_millis(10), None)?;

    Ok(temp_file)
}

fn create_backup_archive(
    output_path: &Path,
    include_paths: &[PathBuf],
    sqlite_dump: Option<&NamedTempFile>,
    age_public_key: &str,
) -> anyhow::Result<()> {
    let file = File::create(output_path)
        .with_context(|| format!("Failed to create output file at '{}'", output_path.display()))?;

    let recipient = age::x25519::Recipient::from_str(age_public_key)
        .map_err(|e| anyhow::anyhow!("Invalid age public key: {e}"))?;

    let encryptor = age::Encryptor::with_recipients(vec![Box::new(recipient)])
        .expect("At least one recipient must be configured");

    let age_writer = encryptor.wrap_output(file)?;
    let zstd_writer = zstd::Encoder::new(age_writer, 3)?;
    let mut tar_builder = tar::Builder::new(zstd_writer);

    for path in include_paths {
        if !path.exists() {
            println!("  [WARNING] Path does not exist, skipping: {}", path.display());
            continue;
        }

        let name_in_archive = path.file_name().unwrap_or(path.as_os_str());

        if path.is_dir() {
            println!("  Archiving directory: {}", path.display());
            tar_builder.append_dir_all(name_in_archive, path)
                .with_context(|| format!("Failed to pack directory '{}'", path.display()))?;
        } else {
            println!("  Archiving file: {}", path.display());
            tar_builder.append_path_with_name(path, name_in_archive)
                .with_context(|| format!("Failed to pack file '{}'", path.display()))?;
        }
    }

    if let Some(dump) = sqlite_dump {
        println!("  Packing consistent SQLite snapshot into archive...");
        tar_builder.append_path_with_name(dump.path(), "database.sqlite3")
            .context("Failed to append SQLite snapshot to archive")?;
    }

    tar_builder.finish()?;
    let zstd_writer = tar_builder.into_inner()?;
    let age_writer = zstd_writer.finish()?;
    let _file = age_writer.finish()?;

    println!("Archive successfully created and encrypted: {}", output_path.display());
    Ok(())
}
