use crate::errors::FileError;
use crate::mrpack::{MRFile, MRIndex, Requirement};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use sha1::{Digest, Sha1};
use sha2::Sha512;
use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::{Read, Write},
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
        let m = MultiProgress::new();

        let pb_files = m.add(ProgressBar::new(self.files.len() as u64));
        pb_files.set_style(
            ProgressStyle::default_bar()
                .template("Server files: [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("=> "),
        );

        for file in &self.files {
            let need_download = match File::open(&file.path) {
                Ok(mut f) => !self.file_is_valid(&mut f, file),
                Err(_) => true,
            };

            if need_download {
                self.download_file(file, &m)?;
            }

            pb_files.inc(1);
        }
        pb_files.finish_and_clear();

        let pb_overrides = ProgressBar::new(self.overrides.len() as u64);
        pb_overrides.set_style(
            ProgressStyle::default_bar()
                .template("Overrides:   [{bar:40.green/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("=> "),
        );

        for (path, content) in &self.overrides {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    create_dir_all(parent)?;
                }
            }
            let mut file = File::create(path)?;
            file.write_all(content)?;
            pb_overrides.inc(1);
        }
        pb_overrides.finish_and_clear();

        Ok(())
    }

    fn download_file(&self, file: &MRFile, m: &MultiProgress) -> Result<(), FileError> {
        if let Some(parent) = Path::new(&file.path).parent() {
            if !parent.exists() {
                create_dir_all(parent)?;
            }
        }

        for url in &file.downloads {
            match self.try_download_file(url, &file.path, m) {
                Ok(()) => return Ok(()),
                Err(_) => continue,
            }
        }

        Err(FileError::AllDownloadsFailed)
    }

    fn try_download_file(
        &self,
        url: &Url,
        path: &PathBuf,
        m: &MultiProgress,
    ) -> Result<(), FileError> {
        let mut response = self.client.get(url.clone()).send()?;
        let total_size = response.content_length().unwrap_or(0);

        let pb_file = m.add(ProgressBar::new(total_size));
        pb_file.set_style(
            ProgressStyle::default_bar()
                .template("Downloading: [{bar:40.green/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("=> "),
        );

        let mut file_handle = File::create(path)?;
        let mut buffer = [0u8; 8192];

        loop {
            let n = response.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            file_handle.write_all(&buffer[..n])?;
            pb_file.inc(n as u64);
        }

        pb_file.finish_and_clear();
        Ok(())
    }

    fn file_is_valid(&self, file: &mut File, mr_file: &MRFile) -> bool {
        let mut data = Vec::with_capacity(file.metadata().map(|md| md.len() as usize).unwrap_or(0));
        file.read_to_end(&mut data).unwrap();

        self.check_sha1(&data, &mr_file.hashes.sha1)
            && self.check_sha512(&data, &mr_file.hashes.sha512)
    }

    fn check_sha1(&self, data: &[u8], expected_hash: &[u8; 20]) -> bool {
        Sha1::digest(data).as_slice() == expected_hash
    }

    fn check_sha512(&self, data: &[u8], expected_hash: &[u8; 64]) -> bool {
        Sha512::digest(data).as_slice() == expected_hash
    }
}
