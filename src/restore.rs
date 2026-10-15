use std::{fs::{self, File}, io::{self, Write}, path::Path, str::FromStr};
use anyhow::Context;

pub fn run_restore(archive_path: &Path, target_dir: &Path, secret_key: &str) -> anyhow::Result<()> {
    let key_content = if Path::new(secret_key).is_file() {
        fs::read_to_string(secret_key)?
    } else {
        secret_key.to_string()
    };

    let identity = age::x25519::Identity::from_str(key_content.trim())
        .map_err(|e| anyhow::anyhow!("Invalid age secret key: {e}"))?;

    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive file at '{}'", archive_path.display()))?;

    let decryptor = match age::Decryptor::new(file)? {
        age::Decryptor::Recipients(d) => d,
        age::Decryptor::Passphrase(_) => anyhow::bail!("Passphrase-encrypted archives are not supported"),
    };

    let decrypted_reader = decryptor.decrypt(std::iter::once(&identity as &dyn age::Identity))
        .context("Decryption failed! Wrong secret key or corrupted archive.")?;

    let zstd_decoder = zstd::Decoder::new(decrypted_reader)
        .context("Failed to initialize Zstandard decompressor")?;

    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create destination directory '{}'", target_dir.display()))?;

    let mut archive = tar::Archive::new(zstd_decoder);
    archive.unpack(target_dir)
        .context("Failed to extract files from tar archive")?;

    println!("Restore completed successfully into '{}'!", target_dir.display());
    Ok(())
}

pub fn prompt_select_backup(files: &[String]) -> anyhow::Result<&str> {
    if files.is_empty() {
        anyhow::bail!("No remote backups found for this job!");
    }

    let reversed_files: Vec<&String> = files.iter().rev().collect();
    println!("\nAvailable cloud backups:");

    for (i, file) in reversed_files.iter().enumerate() {
        let label = if i == 0 { " (latest)" } else { "" };
        println!("  [{}] {}{}", i + 1, file, label);
    }

    print!("\nWhich backup would you like to restore? [1-{}] (Default: 1): ", files.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    let selected_index = if trimmed.is_empty() {
        0
    } else {
        let num: usize = trimmed
            .parse()
            .context("Invalid input! Please enter a valid number.")?;
        if num < 1 || num > reversed_files.len() {
            anyhow::bail!("Selection {} is out of range! Choose between 1 and {}.", num, reversed_files.len());
        }
        num - 1
    };

    Ok(reversed_files[selected_index])
}
