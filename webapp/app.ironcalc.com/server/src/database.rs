use std::io;

use rocket_db_pools::Connection;

use rocket_db_pools::{sqlx, Database};

#[derive(Database)]
#[database("ironcalc")]
pub struct IronCalcDB(sqlx::SqlitePool);

pub async fn get_model_list_from_db(mut db: Connection<IronCalcDB>) -> Result<Vec<String>, io::Error> {
    let row: Vec<(String, )> = sqlx::query_as("SELECT hash FROM models")
        .fetch_all(&mut **db)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch from the database: {}", e);
            io::Error::other("Internal database error")
        })?;
    Ok(row.into_iter().map(|s| s.0).collect())
}

pub async fn add_model(
    mut db: Connection<IronCalcDB>,
    hash: &str,
    bytes: &[u8],
) -> Result<(), io::Error> {
    sqlx::query("INSERT INTO models (hash, bytes) VALUES (?, ?)")
        .bind(hash)
        .bind(bytes)
        .execute(&mut **db)
        .await
        .map_err(|e| {
            eprintln!("Failed to save to the database: {}", e);
            io::Error::other("Internal database error")
        })?;
    Ok(())
}

pub async fn select_model(
    mut db: Connection<IronCalcDB>,
    hash: &str,
) -> Result<Option<Vec<u8>>, io::Error> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT bytes FROM models WHERE hash = ?")
        .bind(hash)
        .fetch_optional(&mut **db)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch from the database: {}", e);
            io::Error::other("Internal database error")
        })?;
    Ok(row.map(|r| r.0))
}
