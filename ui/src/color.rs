#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgba(&self) -> u32 {
        let a = self.a as u32;
        self.to_rgb() + (a << 24)
    }

    pub fn to_rgb(&self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;

        b + (g << 8) + (r << 16)
    }

    pub fn from_rgba(v: u32) -> Self {
        let b = (v & 0x000000ff) as u8;
        let g = ((v & 0x0000ff00) >> 8) as u8;
        let r = ((v & 0x00ff0000) >> 16) as u8;
        let a = ((v & 0xff000000) >> 24) as u8;
        Self { r, g, b, a }
    }

    pub fn set_alpha(&mut self, v: u8) -> &mut Self {
        self.a = v;
        self
    }
}

impl Color {
    pub const fn white() -> Self {
        Self {
            r: 255,
            g: 253,
            b: 253,
            a: 0xff,
        }
    }

    pub const fn black() -> Self {
        Self {
            r: 0x00,
            g: 0x00,
            b: 0x00,
            a: 0xff,
        }
    }

    pub const fn green() -> Self {
        Self {
            r: 0x00,
            g: 0xff,
            b: 0x00,
            a: 0xff,
        }
    }

    pub const fn dark_green() -> Self {
        Self {
            r: 119,
            g: 148,
            b: 86,
            a: 0xff,
        }
    }

    pub const fn dark_blue() -> Self {
        Self {
            r: 0,
            g: 81,
            b: 101,
            a: 0xff,
        }
    }

    pub const fn grey(v: u8) -> Self {
        Self {
            r: v,
            g: v,
            b: v,
            a: 0xff,
        }
    }

    pub const fn light_grey() -> Self {
        Self::grey(0x90)
    }

    pub const fn dark_grey() -> Self {
        Self::grey(0x50)
    }

    pub const fn cream() -> Self {
        Self {
            r: 234,
            g: 236,
            b: 208,
            a: 0xff,
        }
    }

    pub const fn red() -> Self {
        Self {
            r: 241,
            g: 96,
            b: 62,
            a: 0xff,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_is_created_from_rgb() {
        let c = Color::new(0xfe, 0xfd, 0xfc, 0x05);
        let c1 = Color::from_rgba(c.to_rgba());
        assert_eq!(c1, c);
    }
}
