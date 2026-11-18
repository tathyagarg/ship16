pub const fn build_palette() -> [[u8; 3]; 256] {
    let mut palette = [[0u8; 3]; 256];
    let mut i = 0;
    while i < 256 {
        let r = ((i >> 5) & 0x07) * 255 / 7;
        let g = ((i >> 2) & 0x07) * 255 / 7;
        let b = (i & 0x03) * 255 / 3;
        palette[i] = [r as u8, g as u8, b as u8];
        i += 1;
    }

    palette
}

pub static PALETTE: [[u8; 3]; 256] = build_palette();

pub fn to_ansi256(c: u8) -> u8 {
    let [r, g, b] = PALETTE[c as usize];
    let q = |v: u8| ((v as u16 * 5 + 127) / 255) as u8;

    16 + q(r) * 36 + q(g) * 6 + q(b)
}
