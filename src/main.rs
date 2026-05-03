use crate::hald_generator::HaldImageRgbMap;

mod hald_generator;

fn main() {
    let map=HaldImageRgbMap::new(8);
    map.generate_HALD();
}
