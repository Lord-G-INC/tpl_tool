use std::io::{Seek, SeekFrom};

pub const fn align_u32(num: u32) -> u32 {
    (num + 0x1F) & !0x1F
}

pub const fn align_u64(num: u64) -> u64 {
    (num + 0x1F) & ! 0x1F
}

pub trait Algin : Seek {
    fn align_to(&mut self) -> std::io::Result<u64> {
        let pos = self.stream_position()?;
        let algined = align_u64(pos);
        self.seek(SeekFrom::Start(algined))
    }
}

impl<S: Seek> Algin for S {}