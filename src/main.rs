#![allow(dead_code)]

mod types;
mod align;
mod conversion;
mod decoding;

use types::{TPL, TextureFormat, MinFilter, MagFilter, WrapMode};
use binrw::prelude::*;
use std::error::Error;
use std::io::Cursor;
use std::path::PathBuf;
use clap::*;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file(s), providing -o will either name the output name that or extension that depeding on
    /// how many inputs there are.
    #[arg(required = true)]
    pub inputs: Vec<String>,
    #[arg(short)]
    /// Output File/Extension. If there is more than one input, it'll just swap extension
    pub output: Option<PathBuf>,
    /// Texture format for the TPL. C4, C8, and C14X2 do not work
    #[arg(short, default_value_t = TextureFormat::RGB5A3)]
    pub texture_format: TextureFormat,
    /// Min Filter mode for the TPL
    #[arg(short = 'm', default_value_t = MinFilter::Linear)]
    pub min: MinFilter,
    /// Mag Filter mode for the TPL.
    #[arg(short = 'M', default_value_t = MagFilter::Linear)]
    pub mag: MagFilter,
    /// Wrap mode S for the TPL
    #[arg(short = 'S', default_value_t = WrapMode::Clamp)]
    pub wrap_mode_s: WrapMode,
    /// Wrap mode T for the TPL
    #[arg(short = 'T', default_value_t = WrapMode::Clamp)]
    pub wrap_mode_t: WrapMode
}

fn parse_input(input: PathBuf, output: Option<PathBuf>, texture_format: TextureFormat, min: MinFilter, mag: MagFilter, wrap_mode_s: WrapMode, wrap_mode_t: WrapMode) -> Result<(), Box<dyn Error>> {
    if let Some(ext) = input.extension() {
        let ext = String::from(ext.to_string_lossy());
        let output = match ext.as_str() {
            "tpl" => output.unwrap_or(input.with_extension("png")),
            _ => output.unwrap_or(input.with_extension("tpl"))
        };
        if ext == "tpl" {
            let mut cursor = Cursor::new(std::fs::read(input)?);
            let tpl = TPL::read_info(&mut cursor)?;
            if let Some(image) = tpl.get_image(0) {
                image.save(output)?;
            }
        } else {
            let tpl = TPL::from_path(input, texture_format, min, mag, wrap_mode_s, wrap_mode_t)?;
            let data = tpl.into_bytes()?;
            std::fs::write(output, data)?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let Args { inputs, output, texture_format, mag,
        min, wrap_mode_s, wrap_mode_t } = args;
    if inputs.len() == 1 {
        let input = &inputs[0];
        if input.starts_with('*') {
            let paths = glob::glob(&input)?;
            for path in paths.flatten() {
                parse_input(path, output.clone(), texture_format, min, mag, wrap_mode_s, wrap_mode_t)?;
            }
        } else {
            let input = PathBuf::from(input);
            parse_input(input, output.clone(), texture_format, min, mag, wrap_mode_s, wrap_mode_t)?;
        }
        
    } else {
        for input in inputs {
            let input = PathBuf::from(input);
            parse_input(input, output.clone(), texture_format, min, mag, wrap_mode_s, wrap_mode_t)?;
        }
    }
    Ok(())
}
