use std::{
    cell::OnceCell,
    env::join_paths,
    fs::File,
    io::{BufReader, Read},
    sync::OnceLock,
};

use chrono::{Datelike, NaiveDate};
use serde_json::Value;

const MONTH_CODES: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'H', 'L', 'M', 'P', 'R', 'S', 'T'];
const VOWELS: [char; 6] = ['A', 'E', 'I', 'O', 'U', ' '];
const CONSONANTS: [char; 22] = [
    'B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X',
    'Y', 'Z', ' ',
];

static MUNICIPALITIES: OnceLock<Value> = OnceLock::new();

#[derive(Default, PartialEq, Eq)]
enum Gender {
    #[default]
    Male,
    Female,
}

#[derive(Default)]
struct Subject {
    first_name: String,
    last_name: String,
    birth_date: NaiveDate,
    gender: Gender,
    birth_place: String,
    birth_province: String,
}

impl Subject {
    fn last_name_code(&self) -> String {
        let consonants = self.last_name.to_ascii_uppercase().replace(VOWELS, "");
        let vowels = self.last_name.to_ascii_uppercase().replace(CONSONANTS, "");

        format!("{consonants}{vowels}XXX")[..3].to_owned()
    }

    fn first_name_code(&self) -> String {
        let consonants = self.first_name.to_ascii_uppercase().replace(VOWELS, "");
        let b = consonants.as_bytes();

        if b.len() > 3 {
            format!("{}{}{}", b[0] as char, b[2] as char, b[3] as char)
        } else {
            let vowels = self.first_name.to_ascii_uppercase().replace(CONSONANTS, "");
            format!("{consonants}{vowels}XXX")[..3].to_owned()
        }
    }

    fn birth_date_code(&self) -> String {
        let mut year = self.birth_date.year().to_string();
        let month = MONTH_CODES[self.birth_date.month0() as usize];
        let mut day = self.birth_date.day();

        if self.gender == Gender::Female {
            day += 40;
        }

        format!(
            "{year_1}{year_0}{month}{day:02}",
            year_0 = year.pop().unwrap(),
            year_1 = year.pop().unwrap()
        )
    }

    fn birth_place_code(&self) -> String {
        let municipalities = MUNICIPALITIES.get_or_init(initialize_municipalities);

        let municipality = self.birth_place.replace(' ', "-").to_ascii_uppercase();
        let province = self.birth_province.to_ascii_uppercase();

        let found = municipalities
            .get(&format!("{municipality}|{province}"))
            .expect("municipality does not exist in the data");

        found
            .get("code")
            .expect("found municipality did not have code field")
            .as_str()
            .unwrap()
            .to_owned()
    }
}

struct CodiceFiscale(String);

impl CodiceFiscale {
    fn parse() -> Result<Self, String> {
        todo!()
    }
}

fn initialize_municipalities() -> Value {
    let file =
        File::open("data/municipalities.json").expect("municipalities data file does not exist");
    let reader = BufReader::new(file);

    let value: Value =
        serde_json::from_reader(reader).expect("municipalities file was not value JSON");

    assert!(value.is_object(), "municipalities was not an object");

    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_name_code_enough_consonants() {
        let sub = Subject {
            last_name: "Rossi".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "RSS");
    }

    #[test]
    fn test_last_name_code_vowels_needed() {
        let sub = Subject {
            last_name: "Bigi".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "BGI");
    }

    #[test]
    fn test_last_name_code_space_inside() {
        let sub = Subject {
            last_name: "De Rossi".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "DRS");
    }

    #[test]
    fn test_last_name_code_short() {
        let sub = Subject {
            last_name: "Yu".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "YUX");
    }

    #[test]
    fn test_first_name_consonants() {
        let sub = Subject {
            first_name: "Massimo".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "MSM");
    }

    #[test]
    fn test_first_name_vowels_needed() {
        let sub = Subject {
            first_name: "Mario".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "MRA");
    }

    #[test]
    fn test_first_name_space_inside() {
        let sub = Subject {
            first_name: "Maria Teresa".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "MTR");
    }

    #[test]
    fn test_first_name_short() {
        let sub = Subject {
            first_name: "Li".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "LIX");
    }

    #[test]
    fn test_birth_date_code() {
        let sub = Subject {
            birth_date: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            ..Default::default()
        };

        assert_eq!(&sub.birth_date_code(), "24T31");
    }

    #[test]
    fn test_birth_date_small_day() {
        let sub = Subject {
            birth_date: NaiveDate::from_ymd_opt(2024, 12, 5).unwrap(),
            ..Default::default()
        };

        assert_eq!(&sub.birth_date_code(), "24T05");
    }

    #[test]
    fn test_birth_date_female() {
        let sub = Subject {
            birth_date: NaiveDate::from_ymd_opt(2024, 12, 5).unwrap(),
            gender: Gender::Female,
            ..Default::default()
        };

        assert_eq!(&sub.birth_date_code(), "24T45");
    }

    #[test]
    fn test_birth_place() {
        let sub = Subject {
            birth_place: "Abano".into(),
            birth_province: "pd".into(),
            ..Default::default()
        };

        assert_eq!(&sub.birth_place_code(), "A001");
    }
}
