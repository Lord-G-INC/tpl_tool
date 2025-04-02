use std::io::Cursor;
use std::path::Path;
use image::RgbaImage;

use gctex::encode;
use crate::types::*;
use crate::align::*;
use binrw::prelude::*;


impl TPL {
    pub fn from_path<A: AsRef<Path>>(path: A, format: TextureFormat, filter: FilterMode, wrap: WrapMode) -> image::ImageResult<Self> {
        let data = std::fs::read(path)?;
        let fmt = image::guess_format(&data)?;
        let image = image::load(Cursor::new(data), fmt)?;
        let image = image.into_rgba8();
        Ok(Self::from_rgba_image(image, format, filter, wrap))
    }
    pub fn from_rgba_image(image: RgbaImage, format: TextureFormat, filter: FilterMode, wrap: WrapMode) -> Self {
        let mut result = Self::default();
        result.header.image_count = 1;
        result.header.image_offset = 0x0C;
        result.header.identifier = 2142000;
        let mut new_image = ImageNode::default();
        new_image.image_header.width = image.width() as _;
        new_image.image_header.height = image.height() as _;
        new_image.image_header.image_data_offset = 64;
        new_image.offset.image_header_offset = 20;
        new_image.image_header.format = format;
        new_image.image_header.min_filter = filter;
        new_image.image_header.mag_filter = filter;
        new_image.image_header.wrap_s = wrap;
        new_image.image_header.wrap_t = wrap;
        let encoded = 
        encode(new_image.image_header.image_format(), image.as_raw(), image.width(), image.height());
        result.image_datas.insert(64, encoded);
        result.nodes.push(new_image);
        result
    }
    pub fn write_into<W: BinWriterExt>(&self, writer: &mut W) -> BinResult<()> {
        self.header.write_be(writer)?;
        for node in &self.nodes {
            node.offset.write_be(writer)?;
        }
        for node in &self.nodes {
            if let Some(plt) = node.palette_header {
                plt.write_be(writer)?;
            }
        }
        for data in self.palette_datas.values() {
            writer.write(data)?;
        }
        for node in &self.nodes {
            let img = node.image_header;
            img.write_be(writer)?;
        }
        writer.align_to()?;
        for data in self.image_datas.values() {
            writer.write(data)?;
        }
        Ok(())
    }
    pub fn into_bytes(&self) -> BinResult<Vec<u8>> {
        let mut cursor = Cursor::new(vec![]);
        self.write_into(&mut cursor)?;
        Ok(cursor.into_inner())
    }
}