use std::{io::stdout, path::Path, time::SystemTime};

use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use fern::{Dispatch, log_file};
use log::LevelFilter;
use tokio::fs;

use crate::app::create_app;

mod app;
mod components;
mod models;
mod schema;
mod styles;
mod window;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

async fn setup_logger() -> Result<()> {
    // Clear previous log file
    let file = Path::new("photo_manager.log");
    if file.exists() {
        fs::remove_file(file).await?;
    }
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                DateTime::<Utc>::from(SystemTime::now()).format("%T"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(stdout())
        .chain(log_file(file)?)
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    setup_logger().await.expect("Failed to initialize logger");
    create_app().expect("Failed to create window");
}
