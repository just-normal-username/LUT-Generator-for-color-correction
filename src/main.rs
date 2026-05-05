use hald_generator::HaldImageRgbMap;

mod hald_generator;
mod hald_reconstructor;

fn main() {
    let mut map=HaldImageRgbMap::new(8);
    // map.generate_hald_map_squares();
    // map.generate_HALD();
    //map.save_with_grid("/home/matti/Scaricati/Warpinator/hald hp.png".to_string());
    map.hald_from_image("/home/matti/Scaricati/Warpinator/hald hp.png".to_string());
    //map.generate_HALD();
    // //println!("{}", format!("{:?}", map));
    let mut reverse_map=map.reverse_map_squares();
    reverse_map.generate_HALD();
}
