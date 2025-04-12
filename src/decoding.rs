use crate::types::PaletteFormat;

#[inline]
#[must_use]
const fn u8_to_u32_slice(slice: &mut [u8]) -> &mut [u32] {
    unsafe {std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u32, slice.len() / 4)}
}
#[inline]
#[must_use]
const fn u8_to_u16_slice(slice: &[u8]) -> &[u16] {
    unsafe {std::slice::from_raw_parts(slice.as_ptr() as *const u16, slice.len() / 2)}
}

#[inline]
const fn decodepixel_ia8(val: u16) -> u32 {
    let a = (val & 0xFF) as u32;
    let i = (val >> 8) as u32;
    i | (i << 8) | (i << 16) | (a << 24)
}
#[inline]
const fn decodepixel_rgb565(val: u16) -> u32 {
    let r = convert5to8(((val >> 11) & 0x1f) as u8) as u32;
    let g = convert6to8(((val >> 5) & 0x3f) as u8) as u32;
    let b = convert5to8((val & 0x1f) as u8) as u32;
    let a = 0xFFu32;
    r | (g << 8) | (b << 16) | (a << 24)
}
#[inline]
const fn decodepixel_rgb5a3(val: u16) -> u32 {
    let r;
    let g;
    let b;
    let a;
    if val & 0x8000 != 0 {
        r = convert5to8(((val >> 10) & 0x1f) as u8) as u32;
        g = convert5to8(((val >> 5) & 0x1f) as u8) as u32;
        b = convert5to8((val & 0x1f) as u8) as u32;
        a = 0xFFu32;
    } else {
        a = convert3to8(((val >> 12) & 0x7) as u8) as u32;
        r = convert4to8(((val >> 8) & 0xf) as u8) as u32;
        g = convert4to8(((val >> 4) & 0xf) as u8) as u32;
        b = convert4to8((val & 0xf) as u8) as u32;
    }
    r | (g << 8) | (b << 16) | (a << 24)
}
#[inline]
const fn decodepixel_paletted(pixel: u16, format: PaletteFormat) -> u32 {
    match format {
        PaletteFormat::IA8 => decodepixel_ia8(pixel),
        PaletteFormat::RGB565 => decodepixel_rgb565(pixel.swap_bytes()),
        PaletteFormat::RGB5A3 => decodepixel_rgb5a3(pixel.swap_bytes())
    }
}

#[inline]
const fn convert5to8(v: u8) -> u8 {
    (v << 3) | (v >> 2)
}
#[inline]
const fn convert6to8(v: u8) -> u8 {
    (v << 2) | (v >> 4)
}
#[inline]
const fn convert3to8(v: u8) -> u8 {
    (v << 5) | (v << 2) | (v >> 1)
}
#[inline]
const fn convert4to8(v: u8) -> u8 {
    (v << 4) | v
}

#[inline]
fn decodebytesc4(dst: &mut [u32], src: &[u8], tlut: &[u16], tlutformat: PaletteFormat) {
    let mut index = 0;
    for x in 0..4 {
        let val = src[x] as usize;
        let p1 = tlut[val >> 4];
        let p2 = tlut[val & 0xF];
        dst[index] = decodepixel_paletted(p1, tlutformat);
        index += 1;
        dst[index] = decodepixel_paletted(p2, tlutformat);
        index += 1;
    }
}
#[inline]
pub fn decode_texture_c4(dst: &mut [u8], src: &[u8], width: usize, height: usize, tlut: &[u8], 
    tlutformat: PaletteFormat) {
        let wsteps8 = (width + 7) / 8;
        let dst = u8_to_u32_slice(dst);
        let tlut = u8_to_u16_slice(tlut);
        for y in (0..height).step_by(8) {
            let mut ystep = (y / 8) * wsteps8;
            for x in (0..width).step_by(8) {
                let mut xstep = 8 * ystep;
                for iy in 0..8 {
                    decodebytesc4(&mut dst[((y + iy) * width + x)..], &src[(4 * xstep)..], tlut,
                    tlutformat);
                    xstep += 1;
                }
                ystep += 1;
            }
        }
}
#[inline]
fn decodebytesc8(dst: &mut [u32], src: &[u8], tlut: &[u16], tlutformat: PaletteFormat) {
    let mut index = 0;
    for x in 0..8 {
        let val = src[x] as usize;
        dst[index] = decodepixel_paletted(tlut[val], tlutformat);
        index += 1;
    }
}
#[inline]
pub fn decode_texture_c8(dst: &mut [u8], src: &[u8], width: usize, height: usize, tlut: &[u8],
    tlutformat: PaletteFormat) {
        let wsteps8 = (width + 7) / 8;
        let dst = u8_to_u32_slice(dst);
        let tlut = u8_to_u16_slice(tlut);
        for y in (0..height).step_by(4) {
            let mut ystep = (y / 4) * wsteps8;
            for x in (0..width).step_by(8) {
                let mut xstep = 4 * ystep;
                for iy in 0..4 {
                    decodebytesc4(&mut dst[((y + iy) * width + x)..], &src[(8 * xstep)..], tlut,
                    tlutformat);
                    xstep += 1;
                }
                ystep += 1;
            }
        }
}
#[inline]
fn decodebytesc14x2(dst: &mut [u32], src: &[u16], tlut: &[u16], tlutformat: PaletteFormat) {
    let mut index = 0;
    for x in 0..4 {
        let val = (src[x].swap_bytes() & 0x3FFF) as usize;
        dst[index] = decodepixel_paletted(tlut[val], tlutformat);
        index += 1;
    }
}
#[inline]
pub fn decode_texture_c14x2(dst: &mut [u8], src: &[u8], width: usize, height: usize, tlut: &[u8],
    tlutformat: PaletteFormat) {
        let wsteps4 = (width + 3) / 4;
        let dst = u8_to_u32_slice(dst);
        let tlut = u8_to_u16_slice(tlut);
        let src = u8_to_u16_slice(src);
        for y in (0..height).step_by(4) {
            let mut ystep = (y / 4) * wsteps4;
            for x in (0..width).step_by(4) {
                let mut xstep = 4 * ystep;
                for iy in 0..4 {
                    decodebytesc14x2(&mut dst[((y + iy) * width + x)..], &src[(8 * xstep)..], tlut,
                    tlutformat);
                    xstep += 1;
                }
                ystep += 1;
            }
        }
}

