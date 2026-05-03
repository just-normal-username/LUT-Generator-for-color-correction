use image::{ImageBuffer, Rgb, RgbImage};

pub fn generate_HALD() {
    let level:u32 = 8;
    let n:u32 = level.pow(2);
    let size = level.pow(3);
    let mut img = RgbImage::new(size, size);
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

                img.put_pixel(x, y, Rgb([pr, pg, pb]));
            }
        }
    }
    img.save("hald_identity.png").expect("Errore nel salvataggio");
    println!("HALD Identity generata correttamente (512x512).");
}