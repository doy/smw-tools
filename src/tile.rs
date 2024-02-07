use std::io::{Read as _, Seek as _, Write as _};

use image::Pixel as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile([u8; 32]);

impl Tile {
    pub fn load_from_file<F: AsRef<std::path::Path>>(
        filename: F,
        idx: usize,
    ) -> Self {
        let mut fh = std::fs::File::open(filename).unwrap();
        let mut buf = [0; 32];
        fh.seek(std::io::SeekFrom::Start(u64::try_from(idx * 32).unwrap()))
            .unwrap();
        fh.read_exact(&mut buf).unwrap();
        Self(buf)
    }

    pub fn write_to_file<F: AsRef<std::path::Path>>(
        self,
        filename: F,
        idx: usize,
    ) {
        let mut fh = std::fs::OpenOptions::new()
            .write(true)
            .open(filename)
            .unwrap();
        fh.seek(std::io::SeekFrom::Start(u64::try_from(idx * 32).unwrap()))
            .unwrap();
        fh.write_all(&self.0).unwrap();
    }

    pub fn from_image_at(im: &image::GrayImage, idx: usize) -> Self {
        let (width, _) = im.dimensions();
        assert_eq!(width, 128);

        let tile_row = idx / 16;
        let tile_col = idx % 16;

        let mut bytes = [0; 32];
        for row_offset in 0..8 {
            for col_offset in 0..8 {
                let row = tile_row * 8 + row_offset;
                let col = tile_col * 8 + col_offset;
                let pixel = im.get_pixel(
                    col.try_into().unwrap(),
                    row.try_into().unwrap(),
                );
                let val = pixel.channels()[0] / 16;
                let b1 = val & 0x01;
                let b2 = (val >> 1) & 0x01;
                let b3 = (val >> 2) & 0x01;
                let b4 = (val >> 3) & 0x01;
                bytes[row_offset * 2] |= b1 << (7 - col_offset);
                bytes[row_offset * 2 + 1] |= b2 << (7 - col_offset);
                bytes[row_offset * 2 + 16] |= b3 << (7 - col_offset);
                bytes[row_offset * 2 + 17] |= b4 << (7 - col_offset);
            }
        }

        Self(bytes)
    }

    pub fn to_image(self) -> image::GrayImage {
        image::GrayImage::from_fn(8, 8, |x, y| {
            let row = usize::try_from(y).unwrap();
            let col = usize::try_from(x).unwrap();
            image::Luma::from([((self.0[row * 2] >> (7 - col)) & 0x01)
                | (((self.0[row * 2 + 1] >> (7 - col)) & 0x01) << 1)
                | (((self.0[row * 2 + 16] >> (7 - col)) & 0x01) << 2)
                | (((self.0[row * 2 + 17] >> (7 - col)) & 0x01) << 3)])
        })
    }
}
