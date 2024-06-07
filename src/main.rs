use std::{str::FromStr, time::Instant};

use codice_fiscale::{CFString, CodiceFiscale, Gender, NaiveDate, Subject};

fn main() {
    let subject = Subject {
        first_name: CFString::from_str("Giancarlo").unwrap(),
        last_name: CFString::from_str("Galan").unwrap(),
        birth_date: NaiveDate::from_ymd_opt(1956, 9, 10).unwrap(),
        gender: Gender::Male,
        birth_place: CFString::from_str("Padova").unwrap(),
        birth_province: CFString::from_str("PD").unwrap(),
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
