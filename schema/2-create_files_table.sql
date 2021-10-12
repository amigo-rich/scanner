CREATE TABLE files (
       id INTEGER PRIMARY KEY,
       file_path TEXT NOT NULL,
       hash TEXT NOT NULL,
       manifest_id INTEGER NOT NULL,
       FOREIGN KEY (manifest_id) REFERENCES manifest (id)
);
