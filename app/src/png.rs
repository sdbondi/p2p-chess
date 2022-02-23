use crate::sprite::GetRgba;
use crate::Colour;
use png::OutputInfo;

pub struct Png {
    data: Vec<u32>,
    info: OutputInfo,
}

impl Png {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let decoder = png::Decoder::new(bytes);
        let (info, mut reader) = decoder.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;
        let mut data = Vec::new();
        for b in 0..buf.len() / 4 {
            data.push(u32::from_be_bytes([
                buf[b],
                buf[b + 1],
                buf[b + 2],
                buf[b + 3],
            ]));
        }
        Ok(Self { data, info })
    }
}

impl GetRgba for Png {
    fn get_rgba(&self, x: usize, y: usize) -> u32 {
        let idx = x + (y * self.info.width as usize);
        self.data[idx]
    }
}
