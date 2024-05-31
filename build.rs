use std::{collections::HashMap, env, fs::File, io::Write, path::Path};

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = File::create(path).expect("could not create codegen.rs file");

    let active_places = include_str!("data/active_places.csv");
    let inactive_places = include_str!("data/inactive_places.csv");

    let mut active_map = phf_codegen::Map::new();
    let mut inactive_map = phf_codegen::Map::new();

    add_entries(active_places, &mut active_map);
    add_entries(inactive_places, &mut inactive_map);

    writeln!(
        &mut file,
        "pub static ACTIVE_PLACES: phf::Map<&'static str, &'static str> = {};",
        active_map.build()
    )
    .expect("could not write active places map to file");

    writeln!(
        &mut file,
        "pub static INACTIVE_PLACES: phf::Map<&'static str, &'static str> = {};",
        inactive_map.build()
    )
    .expect("could not write inactive places map to file");
}

fn add_entries(places: &'static str, map: &mut phf_codegen::Map<&'static str>) {
    let unique = places.lines().map(parse_entry).collect::<HashMap<_, _>>();

    for (key, value) in unique {
        map.entry(key, &format!(r#""{value}""#));
    }
}

fn parse_entry(line: &str) -> (&str, &str) {
    let (code, rest) = line
        .split_once(',')
        .expect("could not find comma in csv file");

    (rest, code)
}
