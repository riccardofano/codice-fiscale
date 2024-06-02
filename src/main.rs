use std::time::Instant;

use codice_fiscale::{string::CFString, CodiceFiscale, Gender, NaiveDate, Subject};

fn main() {
    let subject = Subject {
        first_name: CFString::new("Giancarlo".into()).unwrap(),
        last_name: CFString::new("Galan".into()).unwrap(),
        birth_date: NaiveDate::from_ymd_opt(1956, 9, 10).unwrap(),
        gender: Gender::Male,
        birth_place: CFString::new("Padova".into()).unwrap(),
        birth_province: CFString::new("PD".into()).unwrap(),
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
