use anyhow::Result;
use duckdb::{Connection, DuckdbConnectionManager};
pub use duckdb;
use tracing::{debug, error};

pub type Pool = r2d2::Pool<DuckdbConnectionManager>;

/// Create a pool of connections to the database at the given `connection_string`
/// with the given `max_size`.
///
/// # Panics
///
/// Panics if `max_size` is set to 0.
pub fn open_db_pool(connection_string: &str, max_size: u32) -> Result<Pool> {
    debug!("Opening database at '{}'", connection_string);

    let manager = DuckdbConnectionManager::file(connection_string)?;
    let pool = r2d2::Pool::builder().max_size(max_size).build(manager)?;

    let mut conn = pool.get()?;

    conn.execute_batch(
        r#"
INSTALL 'json';
LOAD 'json';
"#,
    )?;

    run_migrations(&mut conn)?;

    Ok(pool)
}

const MIGRATION_INDEX_NAME: &str = "migration_index";
const SETTING_TABLE_NAME: &str = "wallowa_setting";

/// Run all migrations that have not yet been run on the given database
fn run_migrations(conn: &mut Connection) -> Result<()> {
    debug!("Running migrations");

    // The full list of migrations to run.
    // Add new migrations to the tail of the vector.
    let migrations = vec![
        // Create the `wallowa_setting` table and initialize the `migration_index`
        r#"
CREATE TABLE IF NOT EXISTS wallowa_setting (
    "name" VARCHAR NOT NULL,
    "value" JSON
);

-- Inserting this row here so that there is no need to upsert as part of `run_migrations`
INSERT INTO wallowa_setting ("name", "value") VALUES ('migration_index', 0);
        "#,
        // Create the `wallowa_raw_data` table
        r#"
CREATE SEQUENCE seq_wallowa_raw_data;
CREATE TABLE IF NOT EXISTS wallowa_raw_data (
    id INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq_wallowa_raw_data'),
    created_at TIMESTAMP DEFAULT now() NOT NULL,
    loaded_at TIMESTAMP,
    "data_source" VARCHAR,
    data_type VARCHAR,
    metadata JSON,
    "data" VARCHAR
);"#,
    ];

    // Start a transaction to wrap all of the migrations
    let tx = conn.transaction()?;

    // First, check whether the `wallowa_setting` table exists.
    // If it doesn't exist, default to the first migration index.
    let settings_exists = tx.query_row(
        r#"
SELECT COUNT(table_name)
FROM information_schema.tables
WHERE table_name = ?"#,
        [SETTING_TABLE_NAME],
        |row| row.get::<_, usize>(0),
    )?;

    let index = if settings_exists == 0 {
        // The settings table doesn't exist so start with the first migration index
        0
    } else {
        // The settings table exists so lookup the migration index.
        // If that setting doesn't exist, start with the first migration index.
        let index_res = tx.query_row(
            &format!(
                "SELECT CAST(value as INTEGER) FROM {} WHERE name = ?",
                SETTING_TABLE_NAME
            ),
            [MIGRATION_INDEX_NAME],
            |row| row.get::<_, usize>(0),
        );
        match index_res {
            Ok(i) => i,
            Err(e) => {
                // On any error, log it and default to the first migration index
                let message = format!("Error loading migration index: {}", e);
                error!(message);
                0
            }
        }
    };

    if index == migrations.len() {
        debug!("No migrations to run");
        return Ok(());
    }

    if index > migrations.len() {
        error!("Unexpected state: `migration_index` of {} is greater than the count of migrations in this version of {}. Possible reasons are running an older version than this database has been used with, a manual update to the `migration_index` value in `setting` table, or something else.",
            index, migrations.len());
    }

    // If 'wallowa_setting' table exists, then run the migrations based on the latest migration version value parsed as a u64.
    // If 'wallawa_setting' table doesn't exist, then assume this is a new database, create the 'wallowa_setting' table, insert the migration version row, and then run migrations.
    for migration in &migrations[index..] {
        debug!(migration = migration, "Running migration `{}`", migration);
        tx.execute_batch(migration)?;
    }

    // Update the `migration_index` to the latest value
    let update_res = tx.execute(
        &format!(
            r#"
UPDATE {}
SET "value" = ?
WHERE "name" = ?;
"#,
            SETTING_TABLE_NAME
        ),
        [&migrations.len().to_string(), MIGRATION_INDEX_NAME],
    );
    match update_res {
        Ok(updated) if updated > 0 => debug!(
            "Updated `{}.{}` to {}",
            SETTING_TABLE_NAME,
            MIGRATION_INDEX_NAME,
            migrations.len()
        ),
        Ok(_) => {
            let msg = format!(
                "Failed to update `{}.{}` to value of {}",
                SETTING_TABLE_NAME,
                MIGRATION_INDEX_NAME,
                migrations.len()
            );
            error!(msg);
        }
        Err(err) => {
            let msg = format!(
                "Failed to update `{}.{}` to value of {} with error {}",
                SETTING_TABLE_NAME,
                MIGRATION_INDEX_NAME,
                migrations.len(),
                err
            );
            error!(msg);
        }
    }

    // Commit the transaction
    Ok(tx.commit()?)
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
