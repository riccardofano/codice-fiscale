use std::time::Instant;

use codice_fiscale::{CodiceFiscale, Gender, NaiveDate, Subject};

fn main() {
    let subject = Subject {
        first_name: "Giancarlo".into(),
        last_name: "Galan".into(),
        birth_date: NaiveDate::from_ymd_opt(1956, 9, 10).unwrap(),
        gender: Gender::Male,
        birth_place: "Padova".into(),
        birth_province: "PD".into(),
    };

    let start = Instant::now();
    let cf = CodiceFiscale::try_from(&subject).unwrap();
    let time = start.elapsed();
    println!("{}, took: {}us", cf.get(), time.as_micros());

    let start = Instant::now();
    let cf = CodiceFiscale::try_from(&subject).unwrap();
    let time = start.elapsed();
    println!("{}, took: {}us", cf.get(), time.as_micros());
}
