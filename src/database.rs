use std::path::Path;

use crate::difference;
use crate::error::Error;
use crate::filemetadata::FileMetadata;
use crate::manifest::{Id, Manifest, Timestamp};
use rusqlite::{params, Connection};

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
    pub fn select_manifests(&self) -> Result<Vec<Manifest>, Error> {
        let sql = r#"
            SELECT id, timestamp, directory_path
            FROM manifest
            ORDER BY id ASC
        "#;
        let mut statement = self.connection.prepare(sql)?;
        let iterator = statement.query_map(params![], |row| {
            Ok(Manifest::from_database(
                Id(row.get(0)?),
                Timestamp(row.get(1)?),
                row.get(2)?,
            ))
        })?;
        let mut results = Vec::new();
        for result in iterator {
            results.push(result?);
        }
        Ok(results)
    }
    pub fn select_manifest(&self, id: &Id) -> Result<Manifest, Error> {
        let sql = r#"
            SELECT id, timestamp, directory_path
            FROM manifest
            WHERE id = ?1
        "#;
        let record = self.connection.query_row(sql, params![id.0], |row| {
            Ok(Manifest::from_database(
                Id(row.get(0)?),
                Timestamp(row.get(1)?),
                row.get(2)?,
            ))
        })?;
        Ok(record)
    }
    pub fn create_manifest_table(
        &mut self,
        timestamp: &Timestamp,
        path: &Path,
    ) -> Result<(), Error> {
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
                    created TEXT,
                    modified TEXT,
                    accessed TEXT,
                    manifest_id INTEGER NOT NULL,
                    FOREIGN KEY (manifest_id) REFERENCES manifest (id)
                )
            "#,
            timestamp.0
        );
        let transaction = self.connection.transaction()?;
        let path = path.to_str().unwrap_or("default");
        transaction.execute(sql, params![timestamp.0, path])?;
        transaction.execute(&create_table_sql, params![])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn delete_manifest_drop_table(&mut self, manifest_id: &Id) -> Result<(), Error> {
        let sql = r#"
            DELETE FROM manifest
            WHERE id = ?1
        "#;
        let manifest_record = self.select_manifest(manifest_id)?;
        let drop_table_sql = format!(
            r#"
                DROP TABLE '{}'
            "#,
            manifest_record.timestamp().0,
        );
        let transaction = self.connection.transaction()?;
        transaction.execute(sql, params![manifest_record.id().0])?;
        transaction.execute(&drop_table_sql, params![])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn insert_file_paths_and_hashes<I>(
        &mut self,
        timestamp: &Timestamp,
        iterator: I,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = FileMetadata>,
    {
        let sql = r#"
            SELECT id
            FROM manifest
            WHERE timestamp = ?1
        "#;
        let transaction = self.connection.transaction()?;
        let insert_sql = format!(
            r#"
                INSERT INTO '{}' (file_path, hash, created, modified, accessed, manifest_id)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            timestamp.0,
        );
        let manifest_id =
            transaction.query_row(sql, params![timestamp.0], |row| Ok(Id(row.get(0)?)))?;
        for file in iterator {
            // Hack for now...probably should be done when scanning or use a u8 vec for path?
            let converted = file.path().to_str().unwrap_or("default");
            transaction.execute(
                &insert_sql,
                params![
                    converted,
                    file.hash(),
                    file.created(),
                    file.modified(),
                    file.accessed(),
                    manifest_id.0
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }
    pub fn select_manifest_differences(
        &self,
        new: &Timestamp,
        old: &Timestamp,
    ) -> Result<Option<Vec<difference::Type>>, Error> {
        let mut differences = Vec::new();
        self.select_hash_differences(new, old, &mut differences)?;
        self.select_removed_paths(new, old, &mut differences)?;
        self.select_added_paths(new, old, &mut differences)?;
        if differences.is_empty() {
            Ok(None)
        } else {
            Ok(Some(differences))
        }
    }
    fn select_hash_differences(
        &self,
        new: &Timestamp,
        old: &Timestamp,
        differences: &mut Vec<difference::Type>,
    ) -> Result<(), Error> {
        let sql = format!(
            r#"
                SELECT
                    n.file_path,
                    n.hash,
                    n.created,
                    n.modified,
                    n.accessed,
                    o.file_path,
                    o.hash,
                    o.created,
                    o.modified,
                    o.accessed
                FROM '{}' AS n
                INNER JOIN '{}' AS o
                ON n.file_path = o.file_path
                WHERE n.hash != o.hash
            "#,
            new.0, old.0
        );
        let mut statement = self.connection.prepare(&sql)?;
        let iterator = statement.query_map(
            params![],
            |row| -> Result<(i64, FileMetadata, i64, FileMetadata), rusqlite::Error> {
                let a = FileMetadata::from_database(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                )
                .unwrap();
                let b = FileMetadata::from_database(
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                )
                .unwrap();
                Ok((old.0, a, new.0, b))
            },
        )?;
        for item in iterator {
            let item = item?;
            let difference = difference::Type::Hash(item.0, item.1, item.2, item.3);
            differences.push(difference);
        }
        Ok(())
    }
    fn select_removed_paths(
        &self,
        new: &Timestamp,
        old: &Timestamp,
        differences: &mut Vec<difference::Type>,
    ) -> Result<(), Error> {
        let sql = format!(
            r#"
                SELECT
                    n.file_path,
                    n.hash,
                    n.created,
                    n.modified,
                    n.accessed
                FROM '{}' AS n
                LEFT JOIN '{}' AS o
                ON n.file_path = o.file_path
                WHERE o.file_path IS NULL
            "#,
            old.0, new.0,
        );
        let mut statement = self.connection.prepare(&sql)?;
        let iterator =
            statement.query_map(params![], |row| -> Result<FileMetadata, rusqlite::Error> {
                Ok(FileMetadata::from_database(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                )
                .unwrap())
            })?;
        for item in iterator {
            let item = item?;
            differences.push(difference::Type::Delete(item));
        }
        Ok(())
    }
    fn select_added_paths(
        &self,
        new: &Timestamp,
        old: &Timestamp,
        differences: &mut Vec<difference::Type>,
    ) -> Result<(), Error> {
        let sql = format!(
            r#"
                SELECT
                    n.file_path,
                    n.hash,
                    n.created,
                    n.modified,
                    n.accessed
                FROM '{}' AS n
                LEFT JOIN '{}' AS o
                ON n.file_path = o.file_path
                WHERE o.file_path IS NULL
            "#,
            new.0, old.0
        );
        let mut statement = self.connection.prepare(&sql)?;
        let iterator =
            statement.query_map(params![], |row| -> Result<FileMetadata, rusqlite::Error> {
                let new = FileMetadata::from_database(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                )
                .unwrap();
                Ok(new)
            })?;
        for item in iterator {
            let item = item?;
            differences.push(difference::Type::Add(item));
        }
        Ok(())
    }
}
