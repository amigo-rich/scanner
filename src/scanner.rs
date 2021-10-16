use crate::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

const READ_MAX: usize = 4098 * 1024;

pub struct Scanner {
    root: PathBuf,
}

fn hash_file_at_path(path: &Path) -> Result<Option<String>, Error> {
    // we don't consider not being able to open a file an error...I
    // don't know if we should or not
    let f = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(None),
        Err(e) => return Err(Error::IO(e)),
    };
    let mut reader = BufReader::with_capacity(READ_MAX, f);

    let mut hasher = blake3::Hasher::new();
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        hasher.update(buffer);
        let len = buffer.len();
        reader.consume(len);
    }
    Ok(Some(hasher.finalize().to_string()))
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
                if let Some(hash) = hash_file_at_path(&path)? {
                    main_send.send((path, hash))?;
                } else {
                    eprintln!("Error reading: {:?}, but continuing..", path);
                }
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
            let dir_iter = match fs::read_dir(path) {
                Ok(readdir) => readdir,
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    eprintln!("Could not open: {:?}, but continuing..", path);
                    return Ok(());
                }
                Err(e) => return Err(Error::IO(e)),
            };
            for entry in dir_iter {
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
