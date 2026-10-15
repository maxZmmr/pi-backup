use std::{fs, path::Path};

use anyhow::Context;
use tempfile::{TempDir};



pub fn upload_and_rotate(
    archive_path: &Path,
    remote: &str,
    job_name: &str,
    keep_last_days: u32,
) -> anyhow::Result<()> {


    if !archive_path.is_file() {
        anyhow::bail!("Watchdog-Alarm: Archivdatei '{}' existiert nicht!", archive_path.display());
    }

    let metadata = fs::metadata(archive_path).with_context(|| format!("Konnte Metadaten von '{}' nicht lesen", archive_path.display()))?;
    let file_size = metadata.len();
    if file_size == 0 {
        anyhow::bail!("Watchdog-Alarm: Archivdatei '{}' ist 0 Bytes groß (korrupt/leer)!", archive_path.display());
    }
    println!("Watchdog OK: Archiv existiert und ist {} Bytes groß.", file_size);


    let destination = format!("{}/{}", remote, job_name);

    println!("Starte upload nach '{}'...", destination);
    let upload_status = std::process::Command::new("rclone")
        .arg("copy")
        .arg(archive_path)
        .arg(&destination)
        .status()
        .context("Fehler beim Ausführen von rclone. Ist rclone im PATH installiert?")?;

    if !upload_status.success() {
        anyhow::bail!("rclone copy fehlgeschlagen mit Status: {:?}", upload_status.code());
    }
    println!("Upload erfolgreich abgeschlossen!");


    let min_age = format!("{}d", keep_last_days);
    println!("Rotiere alte Backups (lösche älter als {})...", min_age);
    let delete_status = std::process::Command::new("rclone")
        .arg("delete")
        .arg(&destination)
        .arg("--min-age")
        .arg(&min_age)
        .status()
        .context("Fehler beim Ausführen von rclone delete")?;

    if !delete_status.success() {
        anyhow::bail!("rclone delete fehlgeschlagen mit Status: {:?}", delete_status.code());
    }

    fs::remove_file(archive_path).with_context(|| format!("Konnte lokales Archiv '{}' nach Upload nicht löschen", archive_path.display()))?;
    println!("Lokales Archiv erfolgreich aufgeräumt.");

  
    Ok(())
}


pub fn list_cloud_backups(remote: &str, job_name: &str) -> anyhow::Result<Vec<String>> {
    let destination = format!("{}/{}", remote, job_name);
    let output = std::process::Command::new("rclone")
        .arg("lsf")
        .arg(&destination)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("rclone Listen Erzeugung fehlgeschlagen mit Status: {:?}", output.status.code());
    }

    let text = String::from_utf8(output.stdout)?;
    let files = text.lines().map(|l| l.trim().to_string()).filter(|line| !line.is_empty()).collect();


    Ok(files)
}


pub fn download_cloud_backup(remote: &str, job_name: &str, file_name: &str) -> anyhow::Result<(TempDir, std::path::PathBuf)> {
    let temp_dir = tempfile::Builder::new().prefix("pi_backup_restore_").tempdir()?;

    let target_file_path = temp_dir.path().join(file_name);

    let source = format!("{}/{}/{}", remote, job_name, file_name);
    let copy_status = std::process::Command::new("rclone")
        .arg("copyto")
        .arg(&source)
        .arg(&target_file_path)
        .status()
        .context("Fehler beim Ausführen von rclone copyto")?;

    if !copy_status.success() {
        anyhow::bail!("rclone Coipy fehlgeschlagen mit Status: {:?}", copy_status.code());
    }
    Ok((temp_dir, target_file_path))
}