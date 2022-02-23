#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgba(&self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;
        let a = self.a as u32;

        b + (g << 8) + (r << 16) + (a << 24)
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

impl Colour {
    pub const fn white() -> Self {
        Self {
            r: 0xff,
            g: 0xff,
            b: 0xff,
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

    pub const fn cream() -> Self {
        Self {
            r: 234,
            g: 236,
            b: 208,
            a: 0xff,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_is_created_from_rgb() {
        let c = Colour::new(0xfe, 0xfd, 0xfc, 0x05);
        let c1 = Colour::from_rgba(c.to_rgba());
        assert_eq!(c1, c);
    }
}
