mod backup;
mod cli;
mod config;
mod notify;
mod restore;
mod upload;

use anyhow::Context;
use backup::run_backup;
use clap::Parser;
use cli::{Cli, Commands};
use config::load_config;
use notify::{send_finish, send_start};
use restore::{prompt_select_backup, run_restore};
use upload::{download_cloud_backup, list_cloud_backups};

use age::secrecy::ExposeSecret;

use crate::config::resolve_config_path;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = resolve_config_path(cli.config.as_deref())?;
    let config = load_config(&config_path)?;

    match cli.command {
        Commands::Backup { job } => {
            println!("Starting backup for job: '{}'", job);
            match config.jobs.get(&job) {
                Some(job_config) => {
                    if let Some(url) = &job_config.healthcheck_url {
                        send_start(url);
                    }

                    let backup_result = run_backup(&job, job_config, &config);

                    if let Some(url) = &job_config.healthcheck_url {
                        if let Err(e) = send_finish(url, &backup_result) {
                            eprintln!("[WARNING] Failed to report status to Healthchecks.io: {e}");
                        }
                    }

                    backup_result?;
                }
                None => {
                    anyhow::bail!("Job '{}' not found in configuration", job);
                }
            }
        }
        Commands::Restore {
            archive,
            job,
            target,
            key,
            latest,
        } => {
            println!("Starting restore into target directory: {:?}", target);

            let (_temp_dir, archive_path) = match archive {
                Some(local_path) => (None, local_path),
                None => {
                    let job_name = match job.as_deref() {
                        Some(name) => name,
                        None => {
                            if config.jobs.len() == 1 {
                                config.jobs.keys().next().unwrap()
                            } else {
                                anyhow::bail!(
                                    "Please specify the job name to restore with --job <NAME>"
                                );
                            }
                        }
                    };

                    println!(
                        "Fetching available backups from cloud for job '{}'...",
                        job_name
                    );
                    let cloud_files = list_cloud_backups(&config.rclone_remote, job_name)?;

                    let selected_file = if latest {
                        cloud_files
                            .last()
                            .context("No backups found in cloud!")?
                            .as_str()
                    } else {
                        prompt_select_backup(&cloud_files)?
                    };

                    println!(
                        "Selected archive: '{}'. Downloading from cloud...",
                        selected_file
                    );
                    let (temp_dir, path) =
                        download_cloud_backup(&config.rclone_remote, job_name, selected_file)?;
                    (Some(temp_dir), path)
                }
            };

            run_restore(&archive_path, &target, &key)?;
        }
        Commands::List { job } => {
            let files = list_cloud_backups(&config.rclone_remote, &job)?;
            println!("Found {} backup(s) on remote storage:", files.len());
            for file in files {
                println!("  - {}", file);
            }
        }
        Commands::Keygen => {
            let key = age::x25519::Identity::generate();
            let pubkey = key.to_public();

            println!("=== Fresh age X25519 Keypair ===");
            println!("Public Key (add to your config.toml):");
            println!("  {}", pubkey);
            println!("\nPrivate Key (store safely in your password manager!):");
            println!("  {}", key.to_string().expose_secret());
        }
    }

    Ok(())
}
