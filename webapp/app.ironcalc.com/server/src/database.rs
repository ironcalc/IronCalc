use std::io;

use std::str::FromStr;

use rocket::fairing::AdHoc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub type IronCalcDB = SqlitePool;

/// Opens the database configured in `databases.ironcalc.url` and manages the pool.
pub fn init() -> AdHoc {
    AdHoc::try_on_ignite("IronCalc database", |rocket| async {
        let url: String = match rocket
            .figment()
            .extract_inner("databases.ironcalc.url")
        {
            Ok(url) => url,
            Err(e) => {
                error!("Missing databases.ironcalc.url: {}", e);
                return Err(rocket);
            }
        };
        let pool = match SqliteConnectOptions::from_str(&url) {
            Ok(options) => SqlitePoolOptions::new().connect_with(options).await,
            Err(e) => Err(e),
        };
        match pool {
            Ok(pool) => Ok(rocket.manage(pool)),
            Err(e) => {
                error!("Failed to open the database: {}", e);
                Err(rocket)
            }
        }
    })
}

pub async fn get_model_list_from_db(db: &IronCalcDB) -> Result<Vec<String>, io::Error> {
    let row: Vec<(String, )> = sqlx::query_as("SELECT * FROM models")
        .fetch_all(db)
        .await
        .unwrap();
    Ok(row.into_iter().map(|s| s.0).collect())
}

pub async fn add_model(
    db: &IronCalcDB,
    hash: &str,
    bytes: &[u8],
) -> Result<(), io::Error> {
    sqlx::query("INSERT INTO models (hash, bytes) VALUES (?, ?)")
        .bind(hash)
        .bind(bytes)
        .execute(db)
        .await
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to save to the database: {}", e),
            )
        })?;
    Ok(())
}

pub async fn select_model(
    db: &IronCalcDB,
    hash: &str,
) -> Result<Vec<u8>, io::Error> {
    let row: (Vec<u8>,) = sqlx::query_as("SELECT bytes FROM models WHERE hash = ?")
        .bind(hash)
        .fetch_one(db)
        .await
        .unwrap();
    Ok(row.0)
}
