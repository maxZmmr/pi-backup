use std::{fs::{self, File}, io::{self, Write}, path::Path, str::FromStr};
use anyhow::Context;



pub fn run_restore(archive_path: &Path, target_dir: &Path, secret_key: &str) -> anyhow::Result<()> {
    let key_content = if Path::new(secret_key).is_file() {
        fs::read_to_string(secret_key)?
    } else {
        secret_key.to_string()
    };
    let identity = age::x25519::Identity::from_str(key_content.trim()).map_err(|e| anyhow::anyhow!("Ungültiger age Secret Key: {e}"))?;


    let file = File::open(archive_path)?;
    let decryptor = match age::Decryptor::new(file)? {
        age::Decryptor::Recipients(d) => d,
        age::Decryptor::Passphrase(_) => anyhow::bail!("Passphrase-Verschlüsselung wird nicht unterstützt"),
    };

    let decrypted_reader = decryptor.decrypt(std::iter::once(&identity as &dyn  age::Identity))
        .context("Entschlüsselung fehlgeschlagen! Falscher Schlüssel oder Datei beschädigt.")?;

    let zstd_decoder = zstd::Decoder::new(decrypted_reader).context("Fehler bei der Zstandard-Dekomprimierung")?;
    let mut archive = tar::Archive::new(zstd_decoder);
    archive.unpack(target_dir).context("Fehler beim Entpacken des Tar-Archivs")?;

    println!("Erfolgreich wiederhergestellt!");

    Ok(())
}


pub fn prompt_select_backup(files: &[String]) -> anyhow::Result<&str> {
    if files.is_empty() {
        anyhow::bail!("Keine Backups für diesen Job in der Cloud gefunden!");
    }

    let reversed_files: Vec<&String> = files.iter().rev().collect();
    println!("\nVerfügbare Cloud-Backups:");

    for (i, file) in reversed_files.iter().enumerate() {
        let label = if i == 0 { " (Neuestes)" } else { "" };
        println!("  [{}] {}{}", i + 1, file, label);
    }

    print!("\nWelches Backup möchtest du wiederherstellen? [1-{}] (Default: 1): ", files.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    
    let selected_index = if trimmed.is_empty() {
        0
    } else {
        let num: usize = trimmed
            .parse()
            .context("Ungültige Eingabe! Bitte eine Zahl eingeben.")?;
        if num < 1 || num > reversed_files.len() {
            anyhow::bail!("Nummer {} ist ungültig! Wähle zwischen 1 und {}.", num, reversed_files.len());
        }
        num - 1
    };
    Ok(reversed_files[selected_index])
}
