use clap::Parser;
use log::{error, info};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the CSV file
    #[clap(
        short,
        long,
        help = "Specify the path to the CSV file containing email credentials."
    )]
    csv_file_path: String,

    /// Old host
    #[clap(
        short,
        long,
        help = "The hostname or IP address of the old email server."
    )]
    old_host: String,

    /// New host
    #[clap(
        short,
        long,
        help = "The hostname or IP address of the new email server."
    )]
    new_host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let args = Args::parse();

    let csv_file_path = &args.csv_file_path;
    let old_host = &args.old_host;
    let new_host = &args.new_host;

    // Read and parse the CSV file
    let mut rdr = csv::Reader::from_path(csv_file_path)?;
    let records: Vec<csv::StringRecord> = rdr.records().collect::<Result<_, _>>()?;

    // Define the number of concurrent tasks
    let concurrency = num_cpus::get() / 4;
    let semaphore = Arc::new(Semaphore::new(concurrency));

    let mut handles = vec![];

    for record in records {
        // Clone the semaphore for each task
        let semaphore = Arc::clone(&semaphore);

        // Extract and validate email credentials from the CSV record
        let old_email = record.get(0).unwrap_or("").to_string();
        let old_password = record.get(1).unwrap_or("").to_string();
        let new_email = record.get(2).unwrap_or("").to_string();
        let new_password = record.get(3).unwrap_or("").to_string();

        // Check if any of the fields are empty and skip the record if so
        if old_email.is_empty()
            || old_password.is_empty()
            || new_email.is_empty()
            || new_password.is_empty()
        {
            error!("Skipping record with missing fields: {:?}", record);
            continue;
        }

        let old_host = old_host.clone();
        let new_host = new_host.clone();

        // Spawn a task to handle the migration
        let handle = task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            match migrate_email(
                &old_host,
                &old_email,
                &old_password,
                &new_host,
                &new_email,
                &new_password,
            )
            .await
            {
                Ok(_) => info!("Successfully migrated {}", old_email),
                Err(e) => error!("Error migrating {}: {}", old_email, e),
            }
        });

        handles.push(handle);
    }

    // Await all spawned tasks and collect the results
    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Task failed: {:?}", e);
        }
    }
    println!("Migration complete.");

    Ok(())
}

async fn migrate_email(
    old_host: &str,
    old_email: &str,
    old_password: &str,
    new_host: &str,
    new_email: &str,
    new_password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("imapsync")
        .arg("--host1")
        .arg(old_host)
        .arg("--user1")
        .arg(old_email)
        .arg("--password1")
        .arg(old_password)
        .arg("--host2")
        .arg(new_host)
        .arg("--user2")
        .arg(new_email)
        .arg("--password2")
        .arg(new_password)
        .arg("--ssl1")
        .arg("--noid")
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to migrate {}: {}",
            old_email,
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}
