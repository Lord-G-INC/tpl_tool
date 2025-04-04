#![allow(dead_code)]

mod types;
mod align;
mod conversion;
mod decoding;

use types::{TPL, TextureFormat, FilterMode, WrapMode};
use binrw::prelude::*;
use std::error::Error;
use std::io::Cursor;
use std::path::PathBuf;
use clap::*;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    pub input: PathBuf,
    pub output: PathBuf,
    #[arg(short, default_value_t = TextureFormat::RGB5A3)]
    pub texture_format: TextureFormat,
    #[arg(short, default_value_t = FilterMode::Linear)]
    pub filter_mode: FilterMode,
    #[arg(short, default_value_t = WrapMode::Clamp)]
    pub wrap_mode: WrapMode
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let Args { input, output, texture_format, filter_mode,
        wrap_mode } = args;
    if let Some(ext) = input.extension() {
        if ext != "tpl" {
            let tpl = TPL::from_path(input, texture_format, filter_mode, wrap_mode)?;
            let data = tpl.into_bytes()?;
            std::fs::write(output, data)?;
        } else if ext == "tpl" {
            let mut cursor = Cursor::new(std::fs::read(input)?);
            let tpl = TPL::read_info(&mut cursor)?;
            if let Some(image) = tpl.get_image(0) {
                image.save(output)?;
            }
        }
    }
    Ok(())
}
