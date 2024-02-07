use std::io::BufRead as _;

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
                    Some("bin") => smw_tools::tile::Tile::load_from_file(
                        &self.from_file,
                        self.from_idx + offset,
                    ),
                    Some("pgm") => {
                        let im =
                            image::io::Reader::open(&self.from_file).unwrap();
                        let im = im.decode().unwrap();
                        let im = image::GrayImage::from(im);
                        smw_tools::tile::Tile::from_image_at(
                            &im,
                            self.from_idx + offset,
                        )
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
