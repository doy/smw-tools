use std::io::{BufRead as _, Read as _, Seek as _, Write as _};

use image::Pixel as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tile([u8; 32]);

impl Tile {
    fn load_from_file<F: AsRef<std::path::Path>>(
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

    fn write_to_file<F: AsRef<std::path::Path>>(
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

    fn from_image_at(im: &image::GrayImage, idx: usize) -> Self {
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

    fn to_image(self) -> image::GrayImage {
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

// Graphics/GFX05.bin:
//   0C Graphics/GFX02.bin:0E:2
#[derive(Debug)]
struct TilemapEdit {
    from_file: std::path::PathBuf,
    from_idx: usize,
    to_file: std::path::PathBuf,
    to_idx: usize,
    size: usize,
}

impl TilemapEdit {
    fn apply(&self) {
        let mut offsets = vec![];
        for x in 0..self.size {
            for y in 0..self.size {
                offsets.push(16 * x + y);
            }
        }
        for offset in offsets {
            let tile =
                match self.from_file.extension().and_then(|s| s.to_str()) {
                    Some("bin") => Tile::load_from_file(
                        &self.from_file,
                        self.from_idx + offset,
                    ),
                    Some("pgm") => {
                        let im =
                            image::io::Reader::open(&self.from_file).unwrap();
                        let im = im.decode().unwrap();
                        let im = image::GrayImage::from(im);
                        Tile::from_image_at(&im, self.from_idx + offset)
                    }
                    _ => unimplemented!(),
                };
            tile.write_to_file(&self.to_file, self.to_idx + offset);
        }
    }
}

#[derive(Debug)]
struct TilemapEdits(Vec<TilemapEdit>);

impl TilemapEdits {
    fn load<P: AsRef<std::path::Path>>(filename: P) -> Self {
        let fh = std::fs::File::open(filename).unwrap();
        let fh = std::io::BufReader::new(fh);
        let rx = regex::Regex::new(
            r"^([0-9a-fA-F]+) (.*\.(?:pgm|bin)):([0-9a-fA-F]+)(?::([124]))$",
        )
        .unwrap();

        let mut current_file = None;
        let mut edits = vec![];
        for line in fh.lines() {
            let line = line.unwrap();
            if line.is_empty() {
                continue;
            }

            if line.starts_with(' ') {
                let line = line.trim_start();
                let cap = rx.captures(line).unwrap();
                edits.push(TilemapEdit {
                    from_file: cap.get(2).unwrap().as_str().into(),
                    from_idx: usize::from_str_radix(
                        cap.get(3).unwrap().as_str(),
                        16,
                    )
                    .unwrap(),
                    to_file: current_file.clone().unwrap(),
                    to_idx: usize::from_str_radix(
                        cap.get(1).unwrap().as_str(),
                        16,
                    )
                    .unwrap(),
                    size: cap
                        .get(4)
                        .map_or(1, |s| s.as_str().parse().unwrap()),
                })
            } else {
                assert!(line.ends_with(".bin:"));
                current_file = Some(line.strip_suffix(':').unwrap().into());
            }
        }

        Self(edits)
    }
}

fn main() {
    let edits_file = std::env::args().nth(1).unwrap();
    let edits = TilemapEdits::load(edits_file);
    for edit in edits.0 {
        edit.apply()
    }
}
