use binrw::prelude::*;
use image::RgbaImage;
use std::io::*;
use std::collections::BTreeMap;
use clap::ValueEnum;
use strum::Display;

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
#[brw(big)]
pub struct FileHeader {
    pub identifier: u32,
    pub image_count: u32,
    pub image_offset: u32
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
#[brw(big)]
pub struct ImageOffset {
    pub image_header_offset: u32,
    pub palette_header_offset: u32
}

impl ImageOffset {
    pub const fn has_palette(&self) -> bool {
        self.palette_header_offset > 0
    }
    pub fn load_palette<R: BinReaderExt>(&self, reader: &mut R) -> BinResult<Option<PaletteHeader>> {
        if !self.has_palette() {
            return Ok(None);
        }
        let pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(self.palette_header_offset as _))?;
        let res = Some(reader.read_be()?);
        reader.seek(SeekFrom::Start(pos))?;
        Ok(res)
    }
    pub fn load_image<R: BinReaderExt>(&self, reader: &mut R) -> BinResult<ImageHeader> {
        let pos = reader.stream_position()?;
        reader.seek(SeekFrom::Start(self.image_header_offset as _))?;
        let res = reader.read_be()?;
        reader.seek(SeekFrom::Start(pos))?;
        Ok(res)
    }
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, ValueEnum, Display)]
#[brw(big, repr = u32)]
#[clap(rename_all = "verbatim")]
pub enum PaletteFormat {
    #[default]
    IA8,
    RGB565,
    RGB5A3
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
#[brw(big)]
pub struct PaletteHeader {
    pub entry_count: u16,
    pub unpacked: u8,
    pub padding: u8,
    pub palette_format: PaletteFormat,
    pub palette_data_offset: u32
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, ValueEnum, Display)]
#[brw(big, repr = u32)]
#[clap(rename_all = "verbatim")]
pub enum WrapMode {
    #[default]
    Clamp,
    Repeat,
    Mirror
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, ValueEnum, Display)]
#[brw(big, repr = u32)]
#[clap(rename_all = "verbatim")]
pub enum FilterMode {
    Nearest,
    #[default]
    Linear,
    // 2 to 5 only works on MinFilter.
    NearestMipmapNearest,
    NearestMipmapLinear,
    LinearMipmapNearest,
    LinearMipmapLinear
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite, ValueEnum, Display)]
#[brw(big, repr = u32)]
#[repr(u32)]
#[clap(rename_all = "verbatim")]
pub enum TextureFormat {
    I4 = 0,
    I8,
    IA4,
    IA8,
    RGB565,
    #[default]
    RGB5A3,
    RGBA8,

    C4 = 0x8,
    C8,
    C14X2,
    CMPR = 0xE
}

#[derive(Debug, Default, Clone, Copy, BinRead, BinWrite)]
#[brw(big)]
pub struct ImageHeader {
    pub height: u16,
    pub width: u16,
    pub format: TextureFormat,
    pub image_data_offset: u32,
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub lod_bias: f32,
    pub edge_lod_enable: u8,
    pub min_lod: u8,
    pub max_lod: u8,
    pub unpacked: u8
}

impl ImageHeader {
    pub fn image_size(&self) -> u32 {
        let format = unsafe {std::mem::transmute(self.format)};
        gctex::compute_image_size(format, self.width as _, self.height as _)
    }
    pub const fn image_format(&self) -> gctex::TextureFormat {
        unsafe {std::mem::transmute(self.format)}
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ImageNode {
    pub offset: ImageOffset,
    pub image_header: ImageHeader,
    pub palette_header: Option<PaletteHeader>
}

impl ImageNode {
    pub fn read_info<R: BinReaderExt>(reader: &mut R) -> BinResult<Self> {
        let mut result = Self::default();
        let Self { offset, image_header, palette_header } 
            = &mut result;
        *offset = reader.read_be()?;
        *image_header = offset.load_image(reader)?;
        *palette_header = offset.load_palette(reader)?;
        Ok(result)
    }
}

#[derive(Debug, Default, Clone)]
pub struct TPL {
    pub header: FileHeader,
    pub nodes: Vec<ImageNode>,
    pub image_datas: BTreeMap<u32, Vec<u8>>,
    pub palette_datas: BTreeMap<u32, Vec<u8>>,
} 

impl TPL {
    pub fn get_node_info(&self, index: usize) -> (ImageHeader, Option<PaletteHeader>) {
        (self.nodes[index].image_header, self.nodes[index].palette_header)
    }
    pub fn get_image(&self, index: usize) -> Option<RgbaImage> {
        let (img, pal) = self.get_node_info(index);
        let texformat = img.image_format();
        let ImageHeader { height, width, .. } = img;
        let (tlutformat, pal_off) = match pal {
            Some(pal) => (pal.palette_format, pal.palette_data_offset),
            None => (PaletteFormat::IA8, 0)
        };
        let src = &self.image_datas[&img.image_data_offset];
        let tlut = self.palette_datas.get(&pal_off).map(Clone::clone).unwrap_or_default();
        let buf = gctex::decode(&src, width as _, height as _, texformat, &tlut, tlutformat as _);
        RgbaImage::from_raw(width as _, height as _, buf)
    }
    pub fn read_info<R: BinReaderExt>(reader: &mut R) -> BinResult<Self> {
        let mut result = Self::default();
        let Self { header, nodes, image_datas, palette_datas } 
        = &mut result;
        *header = reader.read_be()?;
        nodes.reserve_exact(header.image_count as _);
        reader.seek(SeekFrom::Start(header.image_offset as _))?;
        for _ in 0..header.image_count {
            nodes.push(ImageNode::read_info(reader)?);
        }
        for node in nodes.iter() {
            if let Some(palette) = node.palette_header {
                let size = palette.entry_count as usize * 2;
                let mut vec = vec![0u8; size];
                reader.seek(SeekFrom::Start(palette.palette_data_offset as _))?;
                reader.read_exact(&mut vec)?;
                palette_datas.insert(palette.palette_data_offset, vec);
            }
        }
        for node in nodes.iter() {
            reader.seek(SeekFrom::Start(node.image_header.image_data_offset as _))?;
            let size = node.image_header.image_size();
            let mut vec = vec![0u8; size as _];
            reader.read_exact(&mut vec)?;
            image_datas.insert(node.image_header.image_data_offset, vec);
        }
        Ok(result)
    }
}