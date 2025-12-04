use crate::errors::FileError;
use crate::mrpack::MRFile;
use reqwest::blocking::Client;
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::{
    fs::{File, create_dir_all},
    io::Read,
    path::{Path, PathBuf},
};
use url::Url;

pub struct ModManager {
    files: Vec<MRFile>,
    client: Client,
}

impl ModManager {
    pub fn new(files: Vec<MRFile>) -> Self {
        ModManager {
            files,
            client: Client::new(),
        }
    }

    pub fn sync(&self) -> Result<(), FileError> {
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
            println!("Creating directory: {:?}", dir_path.parent());
            create_dir_all(dir_path.parent().unwrap())?;
            println!("Directory created successfully.");
        }

        let mut urls_iter = file.downloads.iter();

        println!("Downloading file: {:?}", file.path);

        loop {
            match urls_iter.next() {
                Some(url) => match self.try_download_file(url, &file.path) {
                    Ok(()) => {
                        println!("File {:?} downloaded successfully.", file.path);
                        break Ok(());
                    }
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
