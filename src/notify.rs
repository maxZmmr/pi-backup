

pub fn send_start(url: &str) {
    let start_url = format!("{}/start", url.trim_end_matches('/'));
    if let Err(e) = ureq::post(&start_url).call() {
        eprintln!("[WARNUNG] Konnte Start-Signal an Healthchecks.io nicht senden: {e}");
    }
}



pub fn send_finish(url: &str, result: &anyhow::Result<()>) -> anyhow::Result<()> {
    let base_url = url.trim_end_matches('/');

    match result {
        Ok(()) => {
            println!("Sende Erfolgs-Ping an Healthchecks.io...");
            ureq::post(base_url).send_string("Backup erfolgreich erstellt, verschlüsselt und in Google Drive gesichert.")?;
        }
        Err(err) => {
            eprintln!("Sende Fehler-Signal an Healthchecks.io...");
            let fail_url = format!("{}/fail", base_url);
            let error_details = format!("Backup fehlgeschlagen:\n{:#}", err);
            
            ureq::post(&fail_url).send_string(&error_details)?;
        }
    }  
    Ok(())
}