use std::{fs::File, io::BufReader, sync::OnceLock};

use chrono::Datelike;
use serde_json::Value;

pub use chrono::NaiveDate;

const MONTH_CODES: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'H', 'L', 'M', 'P', 'R', 'S', 'T'];
const VOWELS: [char; 6] = ['A', 'E', 'I', 'O', 'U', ' '];
const CONSONANTS: [char; 22] = [
    'B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X',
    'Y', 'Z', ' ',
];

static MUNICIPALITIES: OnceLock<Value> = OnceLock::new();

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    Male,
    Female,
}

pub struct Subject {
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDate,
    pub gender: Gender,
    pub birth_place: String,
    pub birth_province: String,
}

const CHECK_CODE_NUM_ODD: [usize; 10] = [1, 0, 5, 7, 9, 13, 15, 17, 19, 21];
const CHECK_CODE_LET_ODD: [usize; 26] = [
    1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6, 8, 12, 14, 16, 10, 22, 25, 24, 23,
];

const CHECK_CODE_NUM_EVEN: [usize; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
const CHECK_CODE_LET_EVEN: [usize; 26] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
];

pub struct CodiceFiscale(String);

impl CodiceFiscale {
    pub fn get(&self) -> &str {
        &self.0
    }

    fn last_name_code(last_name: &str) -> String {
        let consonants = last_name.to_ascii_uppercase().replace(VOWELS, "");
        let vowels = last_name.to_ascii_uppercase().replace(CONSONANTS, "");

        format!("{consonants}{vowels}XXX")[..3].to_owned()
    }

    fn first_name_code(first_name: &str) -> String {
        let consonants = first_name.to_ascii_uppercase().replace(VOWELS, "");
        let b = consonants.as_bytes();

        if b.len() > 3 {
            format!("{}{}{}", b[0] as char, b[2] as char, b[3] as char)
        } else {
            let vowels = first_name.to_ascii_uppercase().replace(CONSONANTS, "");
            format!("{consonants}{vowels}XXX")[..3].to_owned()
        }
    }

    fn birth_date_code(birth_date: NaiveDate, gender: Gender) -> String {
        let mut year = birth_date.year().to_string();
        let month = MONTH_CODES[birth_date.month0() as usize];
        let mut day = birth_date.day();

        if gender == Gender::Female {
            day += 40;
        }

        format!(
            "{year_1}{year_0}{month}{day:02}",
            year_0 = year.pop().unwrap(),
            year_1 = year.pop().unwrap()
        )
    }

    fn birth_place_code(city: &str, province: &str) -> String {
        let municipalities = MUNICIPALITIES.get_or_init(initialize_municipalities);

        let municipality = city.replace(' ', "-").to_ascii_uppercase();
        let province = province.to_ascii_uppercase();

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

    fn compute_checksum(partial_cf: &str) -> char {
        assert_eq!(partial_cf.len(), 15, "expected CF to be 15 characters long");

        let partial_cf = partial_cf.to_uppercase();
        let mut sum = 0;

        // NOTE: This being 2 loops would eliminate the odd/even check
        for (i, c) in partial_cf.bytes().enumerate() {
            // NOTE: The odd/even tables are for 1 indexed numbers so we need to add 1
            sum += match ((i + 1) % 2 == 0, c.is_ascii_digit()) {
                (true, true) => CHECK_CODE_NUM_EVEN[(c - b'0') as usize],
                (true, false) => CHECK_CODE_LET_EVEN[(c - b'A') as usize],
                (false, true) => CHECK_CODE_NUM_ODD[(c - b'0') as usize],
                (false, false) => CHECK_CODE_LET_ODD[(c - b'A') as usize],
            };
        }

        sum %= 26;
        (sum as u8 + b'A') as char
    }
}

impl From<&Subject> for CodiceFiscale {
    fn from(value: &Subject) -> Self {
        let mut output = String::with_capacity(16);

        output.push_str(&Self::last_name_code(&value.last_name));
        output.push_str(&Self::first_name_code(&value.first_name));
        output.push_str(&Self::birth_date_code(value.birth_date, value.gender));
        output.push_str(&Self::birth_place_code(
            &value.birth_place,
            &value.birth_province,
        ));
        output.push(Self::compute_checksum(&output));

        Self(output)
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
        assert_eq!(&CodiceFiscale::last_name_code("Rossi"), "RSS");
    }

    #[test]
    fn test_last_name_code_vowels_needed() {
        assert_eq!(&CodiceFiscale::last_name_code("Bigi"), "BGI");
    }

    #[test]
    fn test_last_name_code_space_inside() {
        assert_eq!(&CodiceFiscale::last_name_code("De Rossi"), "DRS");
    }

    #[test]
    fn test_last_name_code_short() {
        assert_eq!(&CodiceFiscale::last_name_code("Yu"), "YUX");
    }

    #[test]
    fn test_first_name_consonants() {
        assert_eq!(&CodiceFiscale::first_name_code("Massimo"), "MSM");
    }

    #[test]
    fn test_first_name_vowels_needed() {
        assert_eq!(&CodiceFiscale::first_name_code("Mario"), "MRA");
    }

    #[test]
    fn test_first_name_space_inside() {
        assert_eq!(&CodiceFiscale::first_name_code("Maria Teresa"), "MTR");
    }

    #[test]
    fn test_first_name_short() {
        assert_eq!(&CodiceFiscale::first_name_code("Li"), "LIX");
    }

    #[test]
    fn test_birth_date_code() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Male);
        assert_eq!(&res, "24T31");
    }

    #[test]
    fn test_birth_date_small_day() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();

        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Male);
        assert_eq!(&res, "24T05");
    }

    #[test]
    fn test_birth_date_female() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();

        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Female);
        assert_eq!(&res, "24T45");
    }

    #[test]
    fn test_birth_place() {
        let res = CodiceFiscale::birth_place_code("Abano", "PD");
        assert_eq!(&res, "A001");
    }

    #[test]
    fn test_checksum_correct_1() {
        assert_eq!(CodiceFiscale::compute_checksum("RSSMRA70A41F205"), 'Z');
    }

    #[test]
    fn test_checksum_correct_2() {
        assert_eq!(CodiceFiscale::compute_checksum("RSSRRT80A01D229"), 'D');
    }

    #[test]
    fn test_checksum_correct_3() {
        assert_eq!(CodiceFiscale::compute_checksum("GLNGCR56P10G224"), 'Q');
    }

    #[test]
    fn test_checksum_lowercase() {
        assert_eq!(CodiceFiscale::compute_checksum("rssmra70a41f205"), 'Z');
    }

    #[test]
    fn test_encodes_complete_cf_1() {
        let subject = Subject {
            first_name: "Maria".into(),
            last_name: "Rossi".into(),
            birth_date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            gender: Gender::Female,
            birth_place: "Milano".into(),
            birth_province: "Mi".into(),
        };

        assert_eq!(CodiceFiscale::from(&subject).get(), "RSSMRA70A41F205Z");
    }

    #[test]
    fn test_encodes_complete_cf_2() {
        let subject = Subject {
            first_name: "Giancarlo".into(),
            last_name: "Galan".into(),
            birth_date: NaiveDate::from_ymd_opt(1956, 9, 10).unwrap(),
            gender: Gender::Male,
            birth_place: "Padova".into(),
            birth_province: "PD".into(),
        };

        assert_eq!(CodiceFiscale::from(&subject).get(), "GLNGCR56P10G224Q");
    }
}
