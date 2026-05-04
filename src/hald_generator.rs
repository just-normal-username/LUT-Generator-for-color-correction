use std::ops::Add;
use image::{GenericImageView, ImageBuffer, Rgb, RgbImage};
use palette::{Srgb, LinSrgb};

#[derive(Clone)]
#[derive(Debug)]
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
fn from_rgb_to_srgb(px:&Rgb<u8>) -> Srgb {
    Srgb::new(px[0] as f32/255.0, px[1] as f32/255.0, px[2] as f32/255.0)
}
#[derive(Debug)]
pub struct HaldImageRgbMap {
    level:u32,
    map:Vec<Vec<Pixel>>,
}
impl HaldImageRgbMap {
    pub fn new(level: u32) -> Self {
        let size = level.pow(3); // Dimensione lato immagine (es. 512 per level 8)
        HaldImageRgbMap {
            level,
            map: vec![vec![Pixel { r: 0, g: 0, b: 0 }; size as usize]; size as usize],
        }
    }
    pub fn generate_hald_map(&mut self) {
        let n:u32 = self.level.pow(2);
        let size = self.level.pow(3);
        for r in 0..n {
            for g in 0..n {
                for b in 0..n {

                    // 1. Calcoliamo la posizione del tassello (tile) basandoci sul Rosso
                    let tile_x = g/self.level; //1..8
                    let tile_y = b; //1..64

                    // 2. Calcoliamo la coordinata X e Y finale nell'immagine
                    let x = tile_x * n + r;
                    let y = tile_y * self.level + g%self.level;

                    // 3. Normalizziamo i valori per il formato u8 (0..255)
                    // Usiamo (n - 1) come divisore per arrivare esattamente a 255
                    let scale = 255.0 / (n - 1) as f32;

                    let pr = (r as f32 * scale).round() as u8;
                    let pg = ((((g%self.level)*self.level)+(g/self.level)) as f32 * scale).round() as u8;
                    let pb = (b as f32 * scale).round() as u8;
                    self.map[x as usize][y as usize].r=pr;
                    self.map[x as usize][y as usize].g=pg;
                    self.map[x as usize][y as usize].b=pb;
                }
            }
        }
    }
    pub fn generate_hald_map_squares(&mut self) {
        let level = self.level;             // Esempio: 8
        let n = level * level;              // Sfumature per canale: 64
        let size = level * n;               // Dimensione totale: 512

        // Calcoliamo il fattore di scala per mappare 0..63 su 0..255
        let scale = 255.0 / (n - 1) as f32;

        for b in 0..n {
            // Il Blu determina la posizione del tassello nella griglia 8x8
            let tile_x = b % level;
            let tile_y = b / level;

            for g in 0..n {
                for r in 0..n {
                    // Coordinata X: (Posizione tassello * 64) + offset Rosso
                    let x = tile_x * n + r;
                    // Coordinata Y: (Posizione tassello * 64) + offset Verde
                    let y = tile_y * n + g;

                    // Calcolo dei valori RGB lineari
                    let pr = (r as f32 * scale).round() as u8;
                    let pg = (g as f32 * scale).round() as u8;
                    let pb = (b as f32 * scale).round() as u8;

                    // Assegnazione al buffer
                    self.map[x as usize][y as usize].r = pr;
                    self.map[x as usize][y as usize].g = pg;
                    self.map[x as usize][y as usize].b = pb;
                }
            }
        }
    }

    pub fn generate_HALD(&mut self) {
        let lut_size = self.level.pow(3);
        let border = 10;
        let final_size = lut_size + (border * 2);

        let mut img = RgbImage::new(final_size, final_size);

        let background_color = Rgb([255, 255, 255]);
        let marker_color = Rgb([0, 0, 0]);

        for y in 0..final_size {
            for x in 0..final_size {
                // Definiamo i confini esatti della LUT
                let lut_min = border;
                let lut_max = border + lut_size - 1;

                // Logica per l'area della LUT (centrale)
                let is_inside_lut = x >= lut_min && x < (lut_min + lut_size) &&
                    y >= lut_min && y < (lut_min + lut_size);

                if is_inside_lut {
                    let lut_x = x - border;
                    let lut_y = y - border;
                    img.put_pixel(x, y, self.map[lut_x as usize][lut_y as usize].clone().into());
                } else {
                    let mut is_marker = false;

                    // --- LOGICA CROCINI A CONTATTO (Specchiati verso l'interno) ---

                    // Angolo Alto-Sinistra: la punta della L è in (lut_min, lut_min)
                    if (x == lut_min && y < lut_min) || (y == lut_min && x < lut_min) {
                        is_marker = true;
                    }
                    // Angolo Alto-Destra: la punta della L è in (lut_max, lut_min)
                    else if (x == lut_max && y < lut_min) || (y == lut_min && x > lut_max) {
                        is_marker = true;
                    }
                    // Angolo Basso-Sinistra: la punta della L è in (lut_min, lut_max)
                    else if (x == lut_min && y > lut_max) || (y == lut_max && x < lut_min) {
                        is_marker = true;
                    }
                    // Angolo Basso-Destra: la punta della L è in (lut_max, lut_max)
                    else if (x == lut_max && y > lut_max) || (y == lut_max && x > lut_max) {
                        is_marker = true;
                    }

                    if is_marker {
                        img.put_pixel(x, y, marker_color);
                    } else {
                        img.put_pixel(x, y, background_color);
                    }
                }
            }
        }

        img.save("hald_markers.png").expect("Errore nel salvataggio");
        println!("HALD generata: i crocini ora toccano i 4 angoli della LUT.");
    }

    pub fn hald_from_image(& mut self, path: String)->Result<(), String>{
        let img = image::open(path).expect("Impossibile aprire l'immagine");
        let (width, height) = img.dimensions();
        if width != height {
            return Err(format!("Hald image must be squared"));
        }
        // 3. Converti in RGB8 (se non lo è già) per manipolare i pixel facilmente
        let rgb_img = img.to_rgb8();
        let size=self.level.pow(3);
        let pixel_size= width/size;
        let sample_size=pixel_size/2;
        let mut start_x=0;
        let mut start_y=0;
        for x in 0..size{
            for y in 0..size{
                let mut total_linear = LinSrgb::new(0.0, 0.0, 0.0);
                for i in 0..sample_size{
                    for j in 0..sample_size{
                        start_x=pixel_size*x+(pixel_size-sample_size)/2;
                        start_y=pixel_size*y+(pixel_size-sample_size)/2;
                        let px =rgb_img.get_pixel(start_x + i, start_y + j);
                        let srgb_px=from_rgb_to_srgb(px);
                        let linear_px=srgb_px.into_linear();
                        total_linear+=linear_px;
                    }
                }
                let average_linear=total_linear/sample_size.pow(2) as f32;
                let average_rgb:Srgb<u8>=Srgb::from_linear(average_linear);
                self.map[x as usize][y as usize].r=average_rgb.red;
                self.map[x as usize][y as usize].b=average_rgb.blue;
                self.map[x as usize][y as usize].g=average_rgb.green;
            }
        }
        Ok(())
    }
    pub fn reverse_map(self)->Self{
        let mut map=HaldImageRgbMap::new(self.level);
        map.generate_hald_map();
        let mut px;
        let mut x2;
        let mut y2;
        let mut rgb;
        for x in 0..self.level.pow(3){
            for y in 0..self.level.pow(3){
                px=self.map[x as usize][y as usize].clone();
                (x2, y2)=xy_from_rgb(px, self.level);
                rgb=rgb_from_xy(x,y, self.level);
                map.map[x2 as usize][y2 as usize].r=rgb.0[0];
                map.map[x2 as usize][y2 as usize].g=rgb.0[1];
                map.map[x2 as usize][y2 as usize].b=rgb.0[2];
            }
        }
        map
    }
    pub fn reverse_map_squares(self)->Self{
        let mut map=HaldImageRgbMap::new(self.level);
        map.generate_hald_map_squares();
        let mut px;
        let mut x2;
        let mut y2;
        let mut rgb;
        for x in 0..self.level.pow(3){
            for y in 0..self.level.pow(3){
                px=self.map[x as usize][y as usize].clone();
                (x2, y2)=xy_from_rgb_squares(px, self.level);
                rgb=rgb_from_xy_squares(x,y, self.level);
                map.map[x2 as usize][y2 as usize].r=rgb.0[0];
                map.map[x2 as usize][y2 as usize].g=rgb.0[1];
                map.map[x2 as usize][y2 as usize].b=rgb.0[2];
            }
        }
        map
    }
}

fn xy_from_rgb(px: Pixel, level: u32) -> (u32, u32) {
    let n = level.pow(2); // Numero di sfumature (es. 64 se level è 8)
    let scale = (n - 1) as f32 / 255.0;

    // 1. Normalizzazione: trasformiamo i valori da 0..255 a 0..(n-1)
    // Es: se px.r è 255, r diventa 63.
    let r = (px.r as f32 * scale).round() as u32;
    let g = (px.g as f32 * scale).round() as u32;
    let b = (px.b as f32 * scale).round() as u32;

    // 2. Calcolo dei tasselli (Tile Logic)
    let tile_x = g / level; // Colonna del tassello
    let tile_y = b;         // Riga del tassello (nella HALD standard b determina la riga)

    // 3. Calcolo delle coordinate finali
    let x = tile_x * n + r;
    let y = tile_y * level + (g % level);

    (x, y)
}

fn rgb_from_xy(x:u32,y:u32, level:u32)->Rgb<u8>{
    let n = level.pow(2); // Numero di sfumature per canale (es. 64)
    let scale = 255.0 / (n - 1) as f32;

    // 1. Ricaviamo R dalla coordinata X
    // Poiché x = tile_x * n + r, allora r è il resto della divisione per n
    let r_idx = x % n;

    // 2. Ricaviamo B dalla coordinata Y
    // Poiché y = tile_y * level + (g % level), e tile_y = b
    // b è il quoziente della divisione per level
    let b_idx = y / level;

    // 3. Ricaviamo G combinando informazioni da X e Y
    // tile_x = g / level  => lo prendiamo da x / n
    // g_rem = g % level   => lo prendiamo da y % level
    let tile_x = x / n;
    let g_rem = y % level;
    let g_idx = tile_x * level + g_rem;

    // 4. Convertiamo gli indici (0..n-1) in valori RGB (0..255)
    Rgb([
        (r_idx as f32 * scale).round() as u8,
        (g_idx as f32 * scale).round() as u8,
        (b_idx as f32 * scale).round() as u8,
    ])
}

fn xy_from_rgb_squares(px: Pixel, level: u32) -> (u32, u32) {
    let n = level * level;              // Sfumature per canale: 64
    let size = level * n;               // Dimensione totale: 512

    let scale = (n - 1) as f32 / 255.0;

    // 1. Normalizzazione: trasformiamo i valori da 0..255 a 0..(n-1)
    // Es: se px.r è 255, r diventa 63.
    let r = (px.r as f32 * scale).round() as u32;
    let g = (px.g as f32 * scale).round() as u32;
    let b = (px.b as f32 * scale).round() as u32;

    // Il Blu determina la posizione del tassello nella griglia 8x8
    let tile_x = b % level;
    let tile_y = b / level;
    // Coordinata X: (Posizione tassello * 64) + offset Rosso
    let x = tile_x * n + r;
    // Coordinata Y: (Posizione tassello * 64) + offset Verde
    let y = tile_y * n + g;
    (x, y)
}
fn rgb_from_xy_squares(x:u32,y:u32, level:u32)->Rgb<u8>{
    let n = level.pow(2); // Numero di sfumature per canale (es. 64)
    let scale = 255.0 / (n - 1) as f32;

    let r_idx = x % n;

    let g_idx = y % n;

    // 3. Ricaviamo G combinando informazioni da X e Y
    // tile_x = g / level  => lo prendiamo da x / n
    // g_rem = g % level   => lo prendiamo da y % level
    let tile_y = y / n;
    let b_rem = x / n;
    let b_idx = tile_y * level + b_rem;

    // 4. Convertiamo gli indici (0..n-1) in valori RGB (0..255)
    Rgb([
        (r_idx as f32 * scale).round() as u8,
        (g_idx as f32 * scale).round() as u8,
        (b_idx as f32 * scale).round() as u8,
    ])
}

