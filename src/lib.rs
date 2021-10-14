use chrono::Local;
mod database;
use database::Database;
mod error;
use error::Error;
pub mod operation;
use operation::Operation;
mod scanner;
use scanner::Scanner;
mod schema;
use schema::read_schemas;
use std::path::Path;

const DB_PATH: &str = "testing.sqlite";
const SCHEMA_DIR: &str = "schema";

pub fn get_database() -> Result<Database, Error> {
    let path = Path::new(DB_PATH);
    let database = match path.is_file() {
        true => Database::open(path)?,
        false => {
            if let Some(schemas) = read_schemas(Path::new(SCHEMA_DIR))? {
                Database::create(path, schemas.iter())?
            } else {
                return Err(Error::NoSchemaFile(Path::new(SCHEMA_DIR).to_path_buf()));
            }
        }
    };
    Ok(database)
}

pub fn run(operation: Operation) -> Result<(), Error> {
    let mut database = get_database()?;
    match operation {
        Operation::DeleteManifest(manifest_id) => {
            database.delete_manifest_drop_table(manifest_id)?;
        }
        Operation::Index(path) => {
            let scanner = Scanner::new(path)?;
            let results = scanner.index()?;
            let manifest = Local::now().timestamp_millis();
            database.create_manifest_table(manifest, scanner.root())?;
            database.insert_file_paths_and_hashes(manifest, results.into_iter())?;
        }
        Operation::List => {
            let manifests = database.select_manifests()?;
            println!("id\ttimestamp\tpath");
            for manifest in manifests {
                println!(
                    "{}\t{}\t{}",
                    manifest.id(),
                    manifest.record().0,
                    manifest.record().1
                );
            }
        }
        Operation::Scan(manifest, path) => {
            let manifest = database.select_manifest(manifest)?;
            let scanner = Scanner::new(path)?;
            let results = scanner.index()?;
            let new_manifest = Local::now().timestamp_millis();
            database.create_manifest_table(new_manifest, scanner.root())?;
            database.insert_file_paths_and_hashes(new_manifest, results.into_iter())?;
            let differences =
                database.select_manifest_differences(new_manifest, manifest.move_record().0)?;
            for difference in differences {
                println!(
                    "{}|{}|{}|{}",
                    difference.0, difference.1, difference.2, difference.3
                );
            }
        }
    }
    Ok(())
}
