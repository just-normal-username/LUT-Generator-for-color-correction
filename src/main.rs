use hald_generator::HaldImageRgbMap;

mod hald_generator;
mod hald_reconstructor;

fn main() {
    let mut map=HaldImageRgbMap::new(8);
    map.generate_hald_map_squares();
    map.generate_HALD();
    // map.hald_from_image("C:/Users/matti/Pictures/Senza titolo-2.png".to_string());
    // //println!("{}", format!("{:?}", map));
    // let mut reverse_map=map.reverse_map();
    // reverse_map.generate_HALD();
}
