use clap::Parser;
use mrpack::MRIndex;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use zip::ZipArchive;

use crate::mod_manager::ModManager;

mod errors;
mod mod_manager;
mod mrpack;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    path: PathBuf,
}

type IndexError = Box<dyn std::error::Error>;

fn read_index_data(zip: &mut ZipArchive<File>) -> Result<Vec<u8>, IndexError> {
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if file.name() == "modrinth.index.json" {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            return Ok(buf);
        }
    }
    Err("modrinth.index.json not found in zip file".into())
}

fn get_index_data(zip_file: &mut ZipArchive<File>) -> Result<MRIndex, IndexError> {
    let index_data = read_index_data(zip_file)?;
    serde_json::from_slice(&index_data).map_err(Into::into)
}

fn main() -> Result<(), IndexError> {
    let args = Args::parse();

    let file = File::open(args.path)?;
    let mut zip_file = ZipArchive::new(file)?;

    let modrinth_index_data = get_index_data(&mut zip_file)?;

    let manager = ModManager::new(modrinth_index_data.files);

    match manager.sync() {
        Ok(_) => println!("Sync completed successfully"),
        Err(err) => println!("Sync failed: {}", err),
    }

    Ok(())
}
