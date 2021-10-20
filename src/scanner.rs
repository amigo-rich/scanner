use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use crate::error::Error;
use crate::filemetadata::FileMetadata;

pub struct Scanner {
    root: PathBuf,
}

enum Message {
    Path(PathBuf),
    File(FileMetadata),
    Failure(PathBuf),
}

impl Scanner {
    pub fn new(root: PathBuf) -> Result<Scanner, Error> {
        Ok(Scanner { root })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn index(&self) -> Result<Vec<FileMetadata>, Error> {
        let (file_send, file_receive) = mpsc::channel::<Message>();
        let (main_send, main_receive) = mpsc::channel::<Message>();

        let path = self.root.clone();
        let scan_main_send = main_send.clone();
        let scan_handle = thread::spawn(move || -> Result<(), Error> {
            Scanner::visit_dir(&path, &file_send, &scan_main_send)?;
            Ok(())
        });

        let hash_handle = thread::spawn(move || -> Result<(), Error> {
            for message in file_receive {
                if let Message::Path(path) = message {
                    // XXX clone
                    if let Ok(file) = FileMetadata::from_pathbuf(path.clone()) {
                        main_send.send(Message::File(file)).unwrap();
                    } else {
                        main_send.send(Message::Failure(path)).unwrap();
                    }
                }
            }
            Ok(())
        });

        // XXX: gag
        let maybe_error = scan_handle.join()?;
        maybe_error?;
        let maybe_error = hash_handle.join()?;
        maybe_error?;

        let mut files: Vec<FileMetadata> = Vec::new();
        for message in main_receive {
            if let Message::File(file) = message {
                files.push(file);
            } else if let Message::Failure(path) = message {
                eprintln!("Could not create metadata for: {}", path.display());
            }
        }
        Ok(files)
    }
    fn visit_dir(
        path: &Path,
        channel: &mpsc::Sender<Message>,
        main_sender: &mpsc::Sender<Message>,
    ) -> Result<(), Error> {
        if path.is_dir() {
            let dir_iter = match fs::read_dir(path) {
                Ok(readdir) => readdir,
                Err(_) => {
                    main_sender
                        .send(Message::Failure(path.to_path_buf()))
                        .unwrap();
                    return Ok(());
                }
            };
            for entry in dir_iter {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    channel.send(Message::Path(path)).unwrap();
                } else {
                    Scanner::visit_dir(&path, channel, main_sender)?;
                }
            }
        }
        Ok(())
    }
}
