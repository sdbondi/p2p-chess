use crate::{Colour, FrameBuffer};
use std::io::Read;

pub struct Sprite {
    buf: Vec<u8>,
    width: u32,
    height: u32,
}
impl Sprite {
    pub fn load<R: Read>(input: R) -> anyhow::Result<Self> {
        let reader = png::Decoder::new(input);
        let (info, mut reader) = reader.read_info()?;
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf)?;
        Ok(Self {
            buf,
            width: info.width,
            height: info.height,
        })
    }

    pub fn draw(&self, buf: &mut FrameBuffer) {
        // let aspect_w = self.width as f32 / 1024.0f32;
        for (i, v) in self.buf.iter().enumerate() {
            let x = i % 1024; //self.width as usize;
            let y = i / 1024; //self.width as usize;
                              // let y = (i as f32 * aspect_w).round() as usize;
            let c = if *v > 1 { Colour::green().to_rgb() } else { 0 };

            if i % 3 == 0 && c > 0 && x < 1024 && y < 768 {
                buf.put_pixel(x / 2, y / 2, c);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;

    #[test]
    fn load() {}
}
