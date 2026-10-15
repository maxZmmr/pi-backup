pub fn send_start(url: &str) {
    let start_url = format!("{}/start", url.trim_end_matches('/'));
    if let Err(e) = ureq::post(&start_url).call() {
        eprintln!("[WARNING] Failed to send start ping to Healthchecks.io: {e}");
    }
}

pub fn send_finish(url: &str, result: &anyhow::Result<()>) -> anyhow::Result<()> {
    let base_url = url.trim_end_matches('/');

    match result {
        Ok(()) => {
            println!("Sending success ping to Healthchecks.io...");
            ureq::post(base_url)
                .send_string("Backup completed successfully, encrypted, and synced to cloud storage.")?;
        }
        Err(err) => {
            eprintln!("Sending failure alert to Healthchecks.io...");
            let fail_url = format!("{}/fail", base_url);
            let error_details = format!("Backup execution failed:\n{:#}", err);

            ureq::post(&fail_url)
                .send_string(&error_details)?;
        }
    }

    Ok(())
}
