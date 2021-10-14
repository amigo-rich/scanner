CREATE TABLE manifest (
	id INTEGER PRIMARY KEY,
	timestamp INTEGER NOT NULL UNIQUE,
	directory_path TEXT NOT NULL
);
