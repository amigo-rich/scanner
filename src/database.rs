use crate::error::Error;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

pub struct Record<T> {
    id: i64,
    record: T,
}

impl<T> Record<T> {
    pub fn new(id: i64, record: T) -> Self {
        Record::<T> { id, record }
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn record(&self) -> &T {
        &self.record
    }
    pub fn move_record(self) -> T {
        self.record
    }
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn create<I, T>(p: &Path, tables: I) -> Result<Self, Error>
    where
        I: Iterator<Item = T>,
        T: AsRef<str>,
    {
        let mut connection = Connection::open(p)?;

        let transaction = connection.transaction()?;
        for table in tables {
            transaction.execute(table.as_ref(), params![])?;
        }
        transaction.commit()?;

        Ok(Database { connection })
    }
    pub fn open(p: &Path) -> Result<Self, Error> {
        if !p.is_file() {
            return Err(Error::NoFile(p.to_path_buf()));
        }
        let connection = Connection::open(p)?;
        Ok(Database { connection })
    }
    pub fn select_manifests(&self) -> Result<Vec<Record<(i64, String)>>, Error> {
        let sql = r#"
            SELECT id, timestamp, directory_path
            FROM manifest
            ORDER BY id ASC
        "#;
        let mut statement = self.connection.prepare(sql)?;
        let iterator = statement.query_map(params![], |row| {
            Ok(Record::new(row.get(0)?, (row.get(1)?, row.get(2)?)))
        })?;
        let mut results = Vec::new();
        for result in iterator {
            results.push(result?);
        }
        Ok(results)
    }
    pub fn select_manifest(&self, id: i64) -> Result<Record<(i64, String)>, Error> {
        let sql = r#"
            SELECT id, timestamp, directory_path
            FROM manifest
            WHERE id = ?1
        "#;
        let record = self.connection.query_row(sql, params![id], |row| {
            Ok(Record::new(row.get(0)?, (row.get(1)?, row.get(2)?)))
        })?;
        Ok(record)
    }
    pub fn create_manifest_table(&mut self, timestamp: i64, path: &Path) -> Result<(), Error> {
        let sql = r#"
            INSERT INTO manifest (timestamp, directory_path)
            VALUES (?1, ?2)
        "#;
        let create_table_sql = format!(
            r#"
                CREATE TABLE '{}' (
                    id INTEGER PRIMARY KEY,
                    file_path TEXT NOT NULL,
                    hash TEXT NOT NULL,
                    manifest_id INTEGER NOT NULL,
                    FOREIGN KEY (manifest_id) REFERENCES manifest (id)
                )
            "#,
            timestamp
        );
        let transaction = self.connection.transaction()?;
        let path = match path.to_str() {
            Some(path) => path,
            None => "default",
        };
        transaction.execute(sql, params![timestamp, path])?;
        transaction.execute(&create_table_sql, params![])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn delete_manifest_drop_table(&mut self, manifest_id: i64) -> Result<(), Error> {
        let sql = r#"
            DELETE FROM manifest
            WHERE id = ?1
        "#;
        let manifest_record = self.select_manifest(manifest_id)?;
        let drop_table_sql = format!(
            r#"
                DROP TABLE '{}'
            "#,
            manifest_record.record.0,
        );
        let transaction = self.connection.transaction()?;
        transaction.execute(sql, params![manifest_record.id()])?;
        transaction.execute(&drop_table_sql, params![])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn insert_file_paths_and_hashes<I>(
        &mut self,
        timestamp: i64,
        iterator: I,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = (PathBuf, String)>,
    {
        let sql = r#"
            SELECT id
            FROM manifest
            WHERE timestamp = ?1
        "#;
        let transaction = self.connection.transaction()?;
        let insert_sql = format!(
            r#"
                INSERT INTO '{}' (file_path, hash, manifest_id)
                VALUES (?1, ?2, ?3)
            "#,
            timestamp,
        );
        let manifest_id: i64 =
            transaction.query_row(sql, params![timestamp], |row| Ok(row.get(0)?))?;
        for pair in iterator {
            // Hack for now...probably should be done when scanning or use a u8 vec for path?
            let value = pair.0.as_os_str();
            let converted = match value.to_str() {
                Some(value) => value,
                None => "default",
            };
            transaction.execute(&insert_sql, params![converted, pair.1, manifest_id])?;
        }
        transaction.commit()?;
        Ok(())
    }
    pub fn select_manifest_differences(
        &self,
        new: i64,
        old: i64,
    ) -> Result<Vec<(String, String, String, String)>, Error> {
        // XXX check old and new in manifest
        let sql = format!(
            r#"
                SELECT n.file_path, n.hash, o.file_path, o.hash
                FROM '{}' AS n
                LEFT JOIN '{}' AS o
                ON n.file_path = o.file_path
                WHERE n.hash != o.hash
            "#,
            new, old
        );
        let mut statement = self.connection.prepare(&sql)?;
        let iterator = statement.query_map(params![], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut results: Vec<(String, String, String, String)> = Vec::new();
        for result in iterator {
            results.push(result?);
        }
        Ok(results)
    }
}
