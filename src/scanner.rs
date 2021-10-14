use crate::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

pub struct Scanner {
    root: PathBuf,
}

impl Scanner {
    pub fn new(root: PathBuf) -> Result<Scanner, Error> {
        Ok(Scanner { root })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn index(&self) -> Result<Vec<(PathBuf, String)>, Error> {
        let (file_send, file_receive) = mpsc::channel::<PathBuf>();
        let (main_send, main_receive) = mpsc::channel::<(PathBuf, String)>();

        let path = self.root.clone();
        let scan_handle = thread::spawn(move || -> Result<(), Error> {
            Scanner::visit_dir(&path, &file_send)?;
            Ok(())
        });

        let hash_handle = thread::spawn(move || -> Result<(), Error> {
            for path in file_receive {
                let content = match fs::read(&path) {
                    Ok(content) => content,
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            eprintln!("Error reading: {:?}, but continuing..", path);
                            continue;
                        }
                        _ => return Err(Error::IO(e)),
                    },
                };
                let hash = blake3::hash(&content);
                main_send.send((path, hash.to_string()))?;
            }
            Ok(())
        });

        // XXX: gag
        let maybe_error = scan_handle.join()?;
        maybe_error?;
        let maybe_error = hash_handle.join()?;
        maybe_error?;

        let mut files: Vec<(PathBuf, String)> = Vec::new();
        for path_hash in main_receive {
            files.push(path_hash);
        }
        Ok(files)
    }
    fn visit_dir(path: &Path, channel: &mpsc::Sender<PathBuf>) -> Result<(), Error> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    channel.send(path)?;
                } else {
                    Scanner::visit_dir(&path, channel)?;
                }
            }
        }
        Ok(())
    }
}
