mod database;
use database::Database;
mod difference;
pub mod error;
use error::Error;
mod filemetadata;
pub mod manifest;
use manifest::Timestamp;
pub mod operation;
use operation::Operation;
mod scanner;
use scanner::Scanner;
mod schema;
use schema::read_schemas;
use std::path::Path;

const DB_PATH: &str = "testing.sqlite";
const SCHEMA_DIR: &str = "schema";

fn display_result<I, T>(iterator: I)
where
    I: Iterator<Item = T>,
    T: std::fmt::Display,
{
    for item in iterator {
        println!("{}", item);
    }
}

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
        Operation::Compare(first, second) => {
            let new_record = database.select_manifest(&first)?;
            let old_record = database.select_manifest(&second)?;
            if let Some(differences) = database
                .select_manifest_differences(&new_record.timestamp(), &old_record.timestamp())?
            {
                display_result(differences.into_iter());
            } else {
                println!("Sets match.");
            }
        }
        Operation::DeleteManifest(manifest_id) => {
            database.delete_manifest_drop_table(&manifest_id)?;
        }
        Operation::Index(path) => {
            let scanner = Scanner::new(path)?;
            let results = scanner.index()?;
            let manifest = Timestamp::now();
            database.create_manifest_table(&manifest, scanner.root())?;
            database.insert_file_paths_and_hashes(&manifest, results.into_iter())?;
        }
        Operation::List => {
            let manifests = database.select_manifests()?;
            println!("id\ttimestamp\tpath");
            display_result(manifests.into_iter());
        }
        Operation::Scan(manifest_id) => {
            let manifest = database.select_manifest(&manifest_id)?;
            let scanner = Scanner::new(manifest.file_path().to_path_buf())?;
            let results = scanner.index()?;
            let new_manifest = Timestamp::now();
            database.create_manifest_table(&new_manifest, scanner.root())?;
            database.insert_file_paths_and_hashes(&new_manifest, results.into_iter())?;
            if let Some(differences) =
                database.select_manifest_differences(&new_manifest, &manifest.timestamp())?
            {
                display_result(differences.into_iter());
            } else {
                println!("Sets match.");
            }
        }
    }
    Ok(())
}
