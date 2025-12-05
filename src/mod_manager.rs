use crate::errors::FileError;
use crate::mrpack::{MRFile, MRIndex, Requirement};
use reqwest::blocking::Client;
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Read,
    path::{Path, PathBuf},
};
use url::Url;

pub struct ModManager {
    files: Vec<MRFile>,
    overrides: HashMap<PathBuf, Vec<u8>>,
    client: Client,
}

impl ModManager {
    pub fn new(index: MRIndex, overrides: HashMap<PathBuf, Vec<u8>>) -> Self {
        ModManager {
            files: index
                .files
                .iter()
                .filter(|f| {
                    f.env
                        .as_ref()
                        .map_or(true, |env| env.server != Requirement::Unsupported)
                })
                .cloned()
                .collect(),
            overrides,
            client: Client::new(),
        }
    }

    pub fn sync(&self) -> Result<(), FileError> {
        println!("Syncing {} server files..", self.files.len());
        for file in &self.files {
            let current_file_result = File::open(&file.path);

            match current_file_result {
                Ok(mut current_file) => {
                    if !self.file_is_valid(&mut current_file, file) {
                        self.delete_file(&file.path)?;
                        self.download_file(&file)?;
                    }
                }
                Err(_) => {
                    self.download_file(&file)?;
                }
            }
        }

        println!("Syncing overrides..");
        for (path, content) in &self.overrides {
            let dir_path = path.parent().unwrap();
            if !dir_path.exists() {
                create_dir_all(dir_path)?;
            }
            let mut file = File::create(path)?;
            std::io::copy(&mut content.as_slice(), &mut file)?;
        }
        println!("Sync complete!");

        Ok(())
    }

    fn try_download_file(&self, url: &Url, path: &PathBuf) -> Result<(), FileError> {
        let mut response = self.client.get(url.clone()).send()?;
        let mut file = File::create(path)?;
        std::io::copy(&mut response, &mut file)?;
        Ok(())
    }

    fn download_file(&self, file: &MRFile) -> Result<(), FileError> {
        let dir_path = Path::new(&file.path);

        if !dir_path.parent().unwrap().exists() {
            create_dir_all(dir_path.parent().unwrap())?;
        }

        let mut urls_iter = file.downloads.iter();

        loop {
            match urls_iter.next() {
                Some(url) => match self.try_download_file(url, &file.path) {
                    Ok(()) => break Ok(()),
                    Err(_) => {
                        println!("Failed to download file {:?} from URL: {}", file.path, url);
                        continue;
                    }
                },
                None => {
                    println!("All download URLs failed for file {:?}", file.path);
                    break Err(FileError::AllDownloadsFailed);
                }
            };
        }
    }

    fn delete_file(&self, path: &PathBuf) -> Result<(), FileError> {
        std::fs::remove_file(path).map_err(|_| FileError::DeleteFailed)?;
        Ok(())
    }

    fn file_is_valid(&self, file: &mut File, mr_file: &MRFile) -> bool {
        let mut file_data: Vec<u8> =
            Vec::with_capacity(file.metadata().map(|md| md.len() as usize).unwrap_or(0));

        file.read_to_end(&mut file_data).unwrap();

        self.check_sha1(&file_data, &mr_file.hashes.sha1)
            && self.check_sha512(&file_data, &mr_file.hashes.sha512)
    }

    fn check_sha1(&self, data: &[u8], expected_hash: &[u8; 20]) -> bool {
        Sha1::digest(data).as_slice() == expected_hash
    }

    fn check_sha512(&self, data: &[u8], expected_hash: &[u8; 64]) -> bool {
        Sha512::digest(data).as_slice() == expected_hash
    }
}
