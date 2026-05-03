use image::{ImageBuffer, Rgb, RgbImage};


#[derive(Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}
impl From<Pixel> for Rgb<u8> {
    fn from(value: Pixel) -> Self {
        Rgb([value.r, value.g, value.b])
    }
}

pub struct HaldImageRgbMap {
    level:u32,
    map:Vec<Vec<Pixel>>,
}
impl HaldImageRgbMap {
    pub fn new(level:u32) -> HaldImageRgbMap {

        let mut ris = HaldImageRgbMap {level, map:vec![vec![Pixel {r:0,g:0,b:0}; level.pow(3) as usize]; level.pow(3) as usize]};
        let n:u32 = level.pow(2);
        let size = level.pow(3);
        for r in 0..n {
            for g in 0..n {
                for b in 0..n {

                    // 1. Calcoliamo la posizione del tassello (tile) basandoci sul Rosso
                    let tile_x = g/level; //1..8
                    let tile_y = b; //1..64

                    // 2. Calcoliamo la coordinata X e Y finale nell'immagine
                    let x = tile_x * n + r;
                    let y = tile_y * level + g%level;

                    // 3. Normalizziamo i valori per il formato u8 (0..255)
                    // Usiamo (n - 1) come divisore per arrivare esattamente a 255
                    let scale = 255.0 / (n - 1) as f32;

                    let pr = (r as f32 * scale).round() as u8;
                    let pg = ((((g%level)*level)+(g/level)) as f32 * scale).round() as u8;
                    let pb = (b as f32 * scale).round() as u8;
                    ris.map[x as usize][y as usize].r=pr;
                    ris.map[x as usize][y as usize].g=pg;
                    ris.map[x as usize][y as usize].b=pb;
                }
            }
        }
        ris
    }

    pub fn generate_HALD(self) {
        let n:u32 = self.level.pow(2);
        let size = self.level.pow(3);
        let mut img = RgbImage::new(size, size);
        for x in 0..size {
            for y in 0..size {
                img.put_pixel(x, y, self.map[x as usize][y as usize].clone().into());
            }
        }
        img.save("hald_identity.png").expect("Errore nel salvataggio");
        println!("HALD Identity generata correttamente");
    }
}

