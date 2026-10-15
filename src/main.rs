mod cli;
mod config;
mod backup;
mod restore;
mod upload;
mod notify;

use clap::Parser;
use cli::{Cli, Commands};
use config::load_config;
use backup::run_backup;
use restore::{prompt_select_backup, run_restore};
use upload::{list_cloud_backups, download_cloud_backup};
use notify::{send_finish, send_start};

use age::secrecy::ExposeSecret;




fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load_config(&cli.config)?;

    match cli.command {
        Commands::Backup { job } => {
            println!("Starte Backup für den Job: '{}'", job);
            match config.jobs.get(&job) {
                Some(job_config) => {
                    if let Some(url) = &job_config.healthcheck_url {
                        send_start(url);
                    }
                    let backup_result = run_backup(&job, job_config, &config);

                    if let Some(url) = &job_config.healthcheck_url {
                        if let Err(e) = send_finish(url, &backup_result) {
                            eprintln!("[WARNUNG] Konnte Healthchecks.io-Status nicht senden: {e}");
                        }
                    }
                    backup_result?;
                }
                None => {
                    anyhow::bail!("Job '{}' nicht gefunden", job)
                }
            }
        }
        Commands::Restore { archive, job, target, key } => {
            println!("Restore gestartet. Zielordner: {:?}", target);
            let (_temp_file, archive_path) = match archive {

                Some(local_path) => (None, local_path),
                None => {
                    let job_name = job.as_deref().unwrap_or("vaultwarden");
                    println!("Lade Backupliste aus der Cloud für Job '{}'...", job_name);
                    let cloud_files = list_cloud_backups(&config.rclone_remote, job_name)?;
                    let selected_file = prompt_select_backup(&cloud_files)?;
                    println!("Ausgewählt: '{}'. Lade aus Google Drive herunter...", selected_file);
                    let (temp_dir, path) = download_cloud_backup(&config.rclone_remote, job_name, selected_file)?;
                    (Some(temp_dir), path)
                }
            };
            run_restore(&archive_path, &target, &key)?;
        }
        Commands::List { job } => {
            let lines = list_cloud_backups(&config.rclone_remote, &job)?;
            println!("Folgende Dateien wurden gefunden: ({})", lines.len());
            for line in lines {
                println!("{}", line)
            }
            
        }
        Commands::Keygen => {
            let key = age::x25519::Identity::generate();
            let pubkey = key.to_public();

            println!("=== Neues age Schlüsselpaar ===");
            println!("Public Key:");
            println!("  {}", pubkey);
            println!("\nPrivate Key:");
            println!("  {}", key.to_string().expose_secret());
        }
    }
    Ok(())
}