use std::ops::Add;
use image::{GenericImageView, ImageBuffer, Rgb, RgbImage};
use palette::{Srgb, LinSrgb};
use palette::encoding::{Linear};

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

macro_rules! check_lut_bounds {
    ($x:expr, $y:expr, $lut_min:expr, $lut_max:expr, $n:expr, $space:expr) => {
        $x >= $lut_min && $x <= $lut_max &&
        $y >= $lut_min && $y <= $lut_max &&
        (($x - $lut_min) % ($n + $space) < $n) &&
        (($y - $lut_min) % ($n + $space) < $n)
    };
}
macro_rules! get_lut_x {
    ($x:expr, $border:expr, $lut_min:expr, $n:expr, $space:expr) => {
        $x - $border - ($x - $lut_min) / ($n + $space) * $space
    };
}

macro_rules! get_lut_y {
    ($y:expr, $border:expr, $lut_min:expr, $n:expr, $space:expr) => {
        $y - $border - ($y - $lut_min) / ($n + $space) * $space
    };
}

macro_rules! try_fill_pixel {
    ($img:expr, $map:expr, $x:expr, $y:expr, $nx:expr, $ny:expr, $border:expr, $lut_min:expr, $lut_max:expr, $n:expr, $space:expr) => {
        if check_lut_bounds!($nx, $ny, $lut_min, $lut_max, $n, $space) {
            let lx = get_lut_x!($nx, $border, $lut_min, $n, $space) as usize;
            let ly = get_lut_y!($ny, $border, $lut_min, $n, $space) as usize;
            $img.put_pixel($x, $y, $map[lx][ly].clone().into());
            true // Indica che abbiamo trovato e scritto un pixel
        } else {
            false
        }
    };
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
        let space = 5;
        let final_size = lut_size + (border * 2) + space*(self.level -1);
        let n =self.level.pow(2);

        let mut img = RgbImage::new(final_size, final_size);

        let background_color = Rgb([255, 255, 255]);
        let marker_color = Rgb([0, 0, 0]);

        for y in 0..final_size {
            for x in 0..final_size {
                // Definiamo i confini esatti della LUT
                let lut_min = border;
                let lut_max = border + lut_size - 1 + space*(self.level - 1);

                // Logica per l'area della LUT (centrale)
                let is_inside_lut = check_lut_bounds!(x,y,lut_min,lut_max,n,space);

                if is_inside_lut {
                    let lut_x = x - border-(x-lut_min)/(n+space)*space;
                    let lut_y = y - border-(y-lut_min)/(n+space)*space;
                    img.put_pixel(x, y, self.map[lut_x as usize][lut_y as usize].clone().into());
                } else {
                    if x >= lut_min && x <= lut_max &&y >= lut_min && y <= lut_max && !(((x-lut_min)%(n+space)<n) && ((y-lut_min)%(n+space)<n)){
                        // Prova distanza 1
                        if      try_fill_pixel!(img, self.map, x, y, x-1, y,   border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x,   y-1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-1, y-1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+1, y,   border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x,   y+1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+1, y+1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+1, y-1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-1, y+1, border, lut_min, lut_max, n, space) {}

                        // Prova distanza 2 (Espansione richiesta)
                        else if try_fill_pixel!(img, self.map, x, y, x-2, y,   border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x,   y-2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-2, y-2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+2, y,   border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x,   y+2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+2, y+2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+2, y-2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-2, y+2, border, lut_min, lut_max, n, space) {}

                        // Casi misti distanza 2 (es: 2 in x, 1 in y) per coprire meglio i buchi diagonali
                        else if try_fill_pixel!(img, self.map, x, y, x-2, y-1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-2, y+1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+2, y-1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+2, y+1, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-1, y-2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+1, y-2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x-1, y+2, border, lut_min, lut_max, n, space) {}
                        else if try_fill_pixel!(img, self.map, x, y, x+1, y+2, border, lut_min, lut_max, n, space) {}

                        else {
                            img.put_pixel(x, y, Rgb([255, 255, 255]));
                        }
                    }
                    else {
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
        }

        img.save("hald_markers.png").expect("Errore nel salvataggio");
        println!("HALD generata: i crocini ora toccano i 4 angoli della LUT.");
    }

    /// Apre l'immagine al percorso `path`, disegna una griglia nera che separa i
    /// tasselli della HALD (layout a "squares") e salva una copia con suffisso
    /// "_grid.png" nello stesso percorso.
    ///
    /// La funzione assume che la LUT occupi un'area quadrata di lato `level^3` e
    /// che i tasselli abbiano dimensione `level^2` (come in generate_hald_map_squares).
    pub fn hald_from_image(&mut self, path: String) -> Result<(), String> {
        let space_ref: u32 = 5;             // spazio tra tasselli nell'immagine 1:1
        let n        = self.level.pow(2);   // entry per lato di un tassello (es. 64)
        let lut_size = self.level.pow(3);   // entry per lato dell'intera LUT (es. 512)

        let img = image::open(path).map_err(|e| format!("Impossibile aprire l'immagine: {}", e))?;
        let (width, height) = img.dimensions();
        if width != height {
            return Err("Hald image must be squared".to_string());
        }
        let rgb_img = img.to_rgb8();

        // L'immagine è SOLO l'area LUT senza bordo:
        //   level tasselli da n px + (level-1) spazi da space_ref px
        let ref_size     = lut_size + space_ref * (self.level - 1);
        let scale        = width as f64 / ref_size as f64;
        let scaled_space = (space_ref as f64 * scale).round() as u32;
        let scaled_n     = (n as f64 * scale).round() as u32;
        let pixel_size   = (scaled_n / n).max(1);
        let sample_size  = ((pixel_size as f32 * 0.95).round() as u32).max(1);

        for lx in 0..lut_size {
            for ly in 0..lut_size {
                // img_x = lx * pixel_size + (lx / n) * scaled_space
                // Ogni volta che lx attraversa un confine di tassello (lx/n aumenta),
                // si aggiunge scaled_space per saltare lo spazio tra i tasselli.
                let img_x = lx * pixel_size + (lx / n) * scaled_space;
                let img_y = ly * pixel_size + (ly / n) * scaled_space;

                let start_x = img_x + (pixel_size - sample_size) / 2;
                let start_y = img_y + (pixel_size - sample_size) / 2;

                let mut total_linear = LinSrgb::new(0.0f32, 0.0f32, 0.0f32);
                let mut count = 0u32;

                for i in 0..sample_size {
                    for j in 0..sample_size {
                        let px_x = start_x + i;
                        let px_y = start_y + j;
                        if px_x < width && px_y < height {
                            let px        = rgb_img.get_pixel(px_x, px_y);
                            let linear_px = from_rgb_to_srgb(px).into_linear();
                            total_linear += linear_px;
                            count        += 1;
                        }
                    }
                }

                if count > 0 {
                    let average_linear = total_linear / count as f32;
                    let average_rgb: Srgb<u8> = Srgb::from_linear(average_linear);
                    self.map[lx as usize][ly as usize].r = average_rgb.red;
                    self.map[lx as usize][ly as usize].g = average_rgb.green;
                    self.map[lx as usize][ly as usize].b = average_rgb.blue;
                }
            }
        }
        Ok(())
    }

    pub fn save_with_grid(&self, path: String) -> Result<String, String> {
        use std::path::Path;

        let space_ref: u32 = 5;
        let n        = self.level.pow(2);
        let lut_size = self.level.pow(3);

        let img = image::open(&path).map_err(|e| format!("Impossibile aprire immagine: {}", e))?;
        let (width, height) = img.dimensions();
        if width != height {
            return Err("Image must be square".to_string());
        }

        let ref_size     = lut_size + space_ref * (self.level - 1);
        let scale        = width as f64 / ref_size as f64;
        let scaled_space = (space_ref as f64 * scale).round() as u32;
        let scaled_n     = (n as f64 * scale).round() as u32;
        let pixel_size   = (scaled_n / n).max(1);

        let mut rgb_img = img.to_rgb8();
        let black = Rgb([0u8, 0u8, 0u8]);

        // Separatori: tra il tassello (i-1) e il tassello i (i = 1..level)
        // Lo spazio inizia a: i * n * pixel_size + (i-1) * scaled_space
        // Il centro dello spazio è a: i * n * pixel_size + (i-1) * scaled_space + scaled_space / 2
        for i in 1..self.level {
            let gap_center = i * n * pixel_size + (i - 1) * scaled_space + scaled_space / 2;

            // Linea verticale — si estende per tutta l'altezza dell'immagine
            for y in 0..height {
                if gap_center < width {
                    rgb_img.put_pixel(gap_center, y, black);
                }
            }
            // Linea orizzontale — si estende per tutta la larghezza dell'immagine
            for x in 0..width {
                if gap_center < height {
                    rgb_img.put_pixel(x, gap_center, black);
                }
            }
        }

        let p        = Path::new(&path);
        let stem     = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "out".to_string());
        let parent   = p.parent().unwrap_or_else(|| Path::new("."));
        let out_file = parent.join(format!("{}_grid.png", stem));
        let out_path = out_file.to_string_lossy().into_owned();

        rgb_img.save(&out_file).map_err(|e| format!("Errore nel salvataggio: {}", e))?;
        Ok(out_path)
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
        // Creiamo la mappa di output (pre-popolata con la HALD standard a quadrati)
        let size=self.level.pow(3);
        let mut map = HaldImageRgbMap::new(self.level);
        let mut linear_map:Vec<Vec<(LinSrgb, u32)>>=vec![vec![(Srgb::new(0,0,0).into_linear(), 0);size as usize]; size as usize];
        map.generate_hald_map_squares();

        // Mappa baseline per confronto: la HALD standard nello stesso layout
        let mut baseline = HaldImageRgbMap::new(self.level);
        baseline.generate_hald_map_squares();

        // Soglia in spazio lineare (assoluta per canale). Se il pixel misurato
        // differisce dalla HALD standard più di questa soglia in uno qualsiasi
        // dei canali, non lo inseriamo nella mappa di output.
        //todo usare oklab per filtrare
        const REVERSE_THRESHOLD: f32 = 1.0;
        // Soglie addizionali nello spazio sRGB (componenti 0..1) - controllo per canale
        const SRGB_THRESH_R: f32 = 0.27;
        const SRGB_THRESH_G: f32 = 0.27;
        const SRGB_THRESH_B: f32 = 0.27;
        let mut inserted = 0usize;
        let mut skipped = 0usize;

        for x in 0..self.level.pow(3) {
            for y in 0..self.level.pow(3) {
                let px = self.map[x as usize][y as usize].clone();

                // Valore atteso nella HALD standard per la stessa posizione (x,y)
                let expected_px = baseline.map[x as usize][y as usize].clone();

                // Confronto nello spazio lineare
                let px_rgb: Rgb<u8> = px.clone().into();
                let exp_rgb: Rgb<u8> = expected_px.into();
                let px_lin = from_rgb_to_srgb(&px_rgb).into_linear();
                let exp_lin = from_rgb_to_srgb(&exp_rgb).into_linear();

                let dr = (px_lin.red - exp_lin.red).abs();
                let dg = (px_lin.green - exp_lin.green).abs();
                let db = (px_lin.blue - exp_lin.blue).abs();

                if dr > REVERSE_THRESHOLD || dg > REVERSE_THRESHOLD || db > REVERSE_THRESHOLD {
                    // Troppo diverso dalla HALD standard: non inseriamo
                    skipped += 1;
                    continue;
                }

                // Controllo addizionale nello spazio sRGB (assoluto per canale)
                let px_srgb = from_rgb_to_srgb(&px_rgb);
                let exp_srgb = from_rgb_to_srgb(&exp_rgb);
                let sdr = (px_srgb.red - exp_srgb.red).abs();
                let sdg = (px_srgb.green - exp_srgb.green).abs();
                let sdb = (px_srgb.blue - exp_srgb.blue).abs();
                if sdr > SRGB_THRESH_R || sdg > SRGB_THRESH_G || sdb > SRGB_THRESH_B {
                    skipped += 1;
                    continue;
                }

                // Altrimenti, mappiamo il pixel nella posizione ricostruita
                let (x2, y2) = xy_from_rgb_squares(px, self.level);

                // Controllo round-trip: il pixel misurato deve essere vicino al colore
                // canonico della destinazione (rgb_from_xy_squares(x2,y2)). Se non lo è,
                // saltiamo l'inserimento.
                let canonical_dest_rgb = rgb_from_xy_squares(x2, y2, self.level);
                let canonical_dest_lin = from_rgb_to_srgb(&canonical_dest_rgb).into_linear();
                let dr_can = (px_lin.red - canonical_dest_lin.red).abs();
                let dg_can = (px_lin.green - canonical_dest_lin.green).abs();
                let db_can = (px_lin.blue - canonical_dest_lin.blue).abs();
                if dr_can > REVERSE_THRESHOLD || dg_can > REVERSE_THRESHOLD || db_can > REVERSE_THRESHOLD {
                    skipped += 1;
                    continue;
                }

                // ulteriore controllo sRGB per il confronto round-trip
                let canonical_srgb = from_rgb_to_srgb(&canonical_dest_rgb);
                if (px_srgb.red - canonical_srgb.red).abs() > SRGB_THRESH_R ||
                   (px_srgb.green - canonical_srgb.green).abs() > SRGB_THRESH_G ||
                   (px_srgb.blue - canonical_srgb.blue).abs() > SRGB_THRESH_B {
                    skipped += 1;
                    continue;
                }

                let rgb = rgb_from_xy_squares(x, y, self.level);

                // Controllo aggiuntivo: confrontiamo il valore che stiamo per inserire
                // con il valore atteso nella HALD standard nella posizione di destinazione
                // (x2, y2). Se sono troppo diversi, non sovrascriviamo.
                let dest_expected_px = baseline.map[x2 as usize][y2 as usize].clone();
                let dest_expected_rgb: Rgb<u8> = dest_expected_px.into();
                let dest_expected_lin = from_rgb_to_srgb(&dest_expected_rgb).into_linear();
                let rgb_lin = from_rgb_to_srgb(&rgb).into_linear();

                let ddr2 = (rgb_lin.red - dest_expected_lin.red).abs();
                let ddg2 = (rgb_lin.green - dest_expected_lin.green).abs();
                let ddb2 = (rgb_lin.blue - dest_expected_lin.blue).abs();

                if ddr2 > REVERSE_THRESHOLD || ddg2 > REVERSE_THRESHOLD || ddb2 > REVERSE_THRESHOLD {
                    // Il valore da inserire è troppo diverso da quello atteso in destinazione
                    skipped += 1;
                    continue;
                }

                // Controllo addizionale nello spazio sRGB per il valore che stiamo per inserire
                let rgb_srgb = from_rgb_to_srgb(&rgb);
                let dest_expected_srgb = from_rgb_to_srgb(&dest_expected_rgb);
                let sdr_ins = (rgb_srgb.red - dest_expected_srgb.red).abs();
                let sdg_ins = (rgb_srgb.green - dest_expected_srgb.green).abs();
                let sdb_ins = (rgb_srgb.blue - dest_expected_srgb.blue).abs();
                if sdr_ins > SRGB_THRESH_R || sdg_ins > SRGB_THRESH_G || sdb_ins > SRGB_THRESH_B {
                    skipped += 1;
                    continue;
                }
                let mut count=linear_map[x2 as usize][y2 as usize].1;
                count+=1;
                linear_map[x2 as usize][y2 as usize].0.red=linear_map[x2 as usize][y2 as usize].0.red+rgb_lin.red;
                linear_map[x2 as usize][y2 as usize].0.green=linear_map[x2 as usize][y2 as usize].0.green+rgb_lin.green;
                linear_map[x2 as usize][y2 as usize].0.blue=linear_map[x2 as usize][y2 as usize].0.blue+rgb_lin.blue;
                linear_map[x2 as usize][y2 as usize].1=count;
                let result_rgb=Srgb::from_linear(linear_map[x2 as usize][y2 as usize].0.clone()/count as f32);
                map.map[x2 as usize][y2 as usize].r = result_rgb.red;
                map.map[x2 as usize][y2 as usize].g = result_rgb.green;
                map.map[x2 as usize][y2 as usize].b = result_rgb.blue;
                inserted += 1;
            }
        }

        println!("reverse_map_squares: inserted {} entries, skipped {} entries", inserted, skipped);
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
