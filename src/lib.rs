pub mod string;

use std::error::Error;

use chrono::Datelike;
pub use chrono::NaiveDate;
use string::CFString;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug)]
pub struct Subject {
    pub first_name: CFString<String>,
    pub last_name: CFString<String>,
    pub birth_date: NaiveDate,
    pub gender: Gender,
    pub birth_place: CFString<String>,
    pub birth_province: CFString<String>,
}

type CFResult<T> = Result<T, CFError>;

#[derive(Debug, Clone)]
pub struct CodiceFiscale(String);

impl CodiceFiscale {
    const VOWELS: [char; 6] = ['A', 'E', 'I', 'O', 'U', ' '];
    #[rustfmt::skip]
    const CONSONANTS: [char; 22] = [ 'B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z', ' ' ];
    const MONTH_CODES: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'H', 'L', 'M', 'P', 'R', 'S', 'T'];
    const CHECK_CODE_NUM_ODD: [usize; 10] = [1, 0, 5, 7, 9, 13, 15, 17, 19, 21];
    const CHECK_CODE_NUM_EVEN: [usize; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    #[rustfmt::skip]
    const CHECK_CODE_LET_ODD: [usize; 26] = [ 1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6, 8, 12, 14, 16, 10, 22, 25, 24, 23 ];
    #[rustfmt::skip]
    const CHECK_CODE_LET_EVEN: [usize; 26] = [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25 ];

    pub fn get(&self) -> &str {
        &self.0
    }

    fn last_name_code(last_name: CFString<&str>) -> String {
        let consonants = last_name.to_ascii_uppercase().replace(Self::VOWELS, "");
        let vowels = last_name.to_ascii_uppercase().replace(Self::CONSONANTS, "");

        format!("{consonants}{vowels}XXX")[..3].to_owned()
    }

    fn first_name_code(first_name: CFString<&str>) -> String {
        let consonants = first_name.to_ascii_uppercase().replace(Self::VOWELS, "");
        let b = consonants.as_bytes();

        if b.len() > 3 {
            format!("{}{}{}", b[0] as char, b[2] as char, b[3] as char)
        } else {
            let vowels = first_name
                .to_ascii_uppercase()
                .replace(Self::CONSONANTS, "");
            format!("{consonants}{vowels}XXX")[..3].to_owned()
        }
    }

    fn birth_date_code(birth_date: NaiveDate, gender: Gender) -> CFResult<String> {
        let year = birth_date.year();
        if year < 1700 {
            return Err(CFError::InvalidYear);
        }

        let month = Self::MONTH_CODES[birth_date.month0() as usize];
        let mut day = birth_date.day();

        if gender == Gender::Female {
            day += 40;
        }

        Ok(format!("{year:02}{month}{day:02}", year = year % 100))
    }

    fn birth_place_code(city: CFString<&str>, province: CFString<&str>) -> Option<String> {
        let municipality = city.replace(' ', "-").to_ascii_lowercase();
        let province = province.to_ascii_uppercase();

        let key = format!("{municipality},{province}");

        if let Some(active_found) = ACTIVE_PLACES.get(&key) {
            return Some(active_found.to_string());
        }

        INACTIVE_PLACES.get(&key).map(|p| p.to_string())
    }

    fn compute_checksum(partial_cf: &str) -> CFResult<char> {
        if partial_cf.len() != 15 {
            return Err(CFError::InvalidChecksumInput);
        }

        let partial_cf = partial_cf.to_uppercase();
        let mut sum = 0;

        // NOTE: This being 2 loops would eliminate the odd/even check
        for (i, c) in partial_cf.bytes().enumerate() {
            // NOTE: The odd/even tables are for 1 indexed numbers so we need to add 1
            sum += match ((i + 1) % 2 == 0, c.is_ascii_digit()) {
                (true, true) => Self::CHECK_CODE_NUM_EVEN[(c - b'0') as usize],
                (true, false) => Self::CHECK_CODE_LET_EVEN[(c - b'A') as usize],
                (false, true) => Self::CHECK_CODE_NUM_ODD[(c - b'0') as usize],
                (false, false) => Self::CHECK_CODE_LET_ODD[(c - b'A') as usize],
            };
        }

        sum %= 26;
        Ok((sum as u8 + b'A') as char)
    }
}

impl TryFrom<&Subject> for CodiceFiscale {
    type Error = CFError;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        let mut output = String::with_capacity(16);

        output.push_str(&Self::last_name_code(value.last_name.as_deref()));
        output.push_str(&Self::first_name_code(value.first_name.as_deref()));
        output.push_str(&Self::birth_date_code(value.birth_date, value.gender)?);

        let place_code = Self::birth_place_code(
            value.birth_place.as_deref(),
            value.birth_province.as_deref(),
        )
        .ok_or(Self::Error::BelfioreCodeNotFound)?;
        output.push_str(&place_code);

        output.push(Self::compute_checksum(&output)?);

        Ok(Self(output))
    }
}

#[derive(Debug)]
pub enum CFError {
    BelfioreCodeNotFound,
    InvalidYear,
    InvalidChecksumInput,
    InvalidString,
}

impl Error for CFError {}
impl std::fmt::Display for CFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::BelfioreCodeNotFound => "could not find belfiore code for this city and province",
            Self::InvalidYear => "the year must be greater than 1700",
            Self::InvalidChecksumInput => "input must be 15 characters long",
            Self::InvalidString => "characters must be alphabetic or a space",
        };

        write!(f, "{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_name_code_enough_consonants() {
        let name = CFString::new("Rossi").unwrap();
        assert_eq!(&CodiceFiscale::last_name_code(name), "RSS");
    }

    #[test]
    fn test_last_name_code_vowels_needed() {
        let name = CFString::new("Bigi").unwrap();
        assert_eq!(&CodiceFiscale::last_name_code(name), "BGI");
    }

    #[test]
    fn test_last_name_code_space_inside() {
        let name = CFString::new("De Rossi").unwrap();
        assert_eq!(&CodiceFiscale::last_name_code(name), "DRS");
    }

    #[test]
    fn test_last_name_code_short() {
        let name = CFString::new("Yu").unwrap();
        assert_eq!(&CodiceFiscale::last_name_code(name), "YUX");
    }

    #[test]
    fn test_first_name_consonants() {
        let name = CFString::new("Massimo").unwrap();
        assert_eq!(&CodiceFiscale::first_name_code(name), "MSM");
    }

    #[test]
    fn test_first_name_vowels_needed() {
        let name = CFString::new("Mario").unwrap();
        assert_eq!(&CodiceFiscale::first_name_code(name), "MRA");
    }

    #[test]
    fn test_first_name_space_inside() {
        let name = CFString::new("Maria Teresa").unwrap();
        assert_eq!(&CodiceFiscale::first_name_code(name), "MTR");
    }

    #[test]
    fn test_first_name_short() {
        let name = CFString::new("Li").unwrap();
        assert_eq!(&CodiceFiscale::first_name_code(name), "LIX");
    }

    #[test]
    fn test_first_name_super_short() {
        let name = CFString::new("W").unwrap();
        assert_eq!(&CodiceFiscale::first_name_code(name), "WXX");
    }

    #[test]
    fn test_birth_date_code() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Male);
        assert_eq!(&res.unwrap(), "24T31");
    }

    #[test]
    fn test_birth_date_small_day() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();

        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Male);
        assert_eq!(&res.unwrap(), "24T05");
    }

    #[test]
    fn test_birth_date_female() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();

        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Female);
        assert_eq!(&res.unwrap(), "24T45");
    }

    #[test]
    fn test_birth_date_ends_with_0_something() {
        let birth_date = NaiveDate::from_ymd_opt(2003, 12, 6).unwrap();
        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Male);
        assert_eq!(&res.unwrap(), "03T06");
    }

    #[test]
    fn test_birth_date_invalid_year() {
        let birth_date = NaiveDate::from_ymd_opt(1508, 4, 12).unwrap();
        let res = CodiceFiscale::birth_date_code(birth_date, Gender::Female);
        assert!(&res.is_err());
    }

    #[test]
    fn test_birth_place() {
        let place = CFString::new("Abano").unwrap();
        let province = CFString::new("PD").unwrap();
        let res = CodiceFiscale::birth_place_code(place, province);
        assert_eq!(res.as_deref(), Some("A001"));
    }

    #[test]
    fn test_birth_place_not_found() {
        let place = CFString::new("I dont exist").unwrap();
        let province = CFString::new("PD").unwrap();
        let res = CodiceFiscale::birth_place_code(place, province);
        assert_eq!(res.as_deref(), None);
    }

    #[test]
    fn test_checksum_correct_1() {
        let res = CodiceFiscale::compute_checksum("RSSMRA70A41F205").unwrap();
        assert_eq!(res, 'Z');
    }

    #[test]
    fn test_checksum_correct_2() {
        let res = CodiceFiscale::compute_checksum("RSSRRT80A01D229").unwrap();
        assert_eq!(res, 'D');
    }

    #[test]
    fn test_checksum_correct_3() {
        let res = CodiceFiscale::compute_checksum("GLNGCR56P10G224").unwrap();
        assert_eq!(res, 'Q');
    }

    #[test]
    fn test_checksum_lowercase() {
        let res = CodiceFiscale::compute_checksum("rssmra70a41f205").unwrap();
        assert_eq!(res, 'Z');
    }

    #[test]
    fn test_checksum_short_input() {
        assert!(CodiceFiscale::compute_checksum("RSSMRA70A41205").is_err());
    }

    #[test]
    fn test_checksum_long_input() {
        assert!(CodiceFiscale::compute_checksum("RSSMRA70A41F205A").is_err());
    }

    #[test]
    fn test_encodes_complete_cf_1() {
        let subject = Subject {
            first_name: CFString::new("Maria".into()).unwrap(),
            last_name: CFString::new("Rossi".into()).unwrap(),
            birth_date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            gender: Gender::Female,
            birth_place: CFString::new("Milano".into()).unwrap(),
            birth_province: CFString::new("Mi".into()).unwrap(),
        };

        assert_eq!(
            CodiceFiscale::try_from(&subject).unwrap().get(),
            "RSSMRA70A41F205Z"
        );
    }

    #[test]
    fn test_encodes_complete_cf_2() {
        let subject = Subject {
            first_name: CFString::new("Giancarlo".into()).unwrap(),
            last_name: CFString::new("Galan".into()).unwrap(),
            birth_date: NaiveDate::from_ymd_opt(1956, 9, 10).unwrap(),
            gender: Gender::Male,
            birth_place: CFString::new("Padova".into()).unwrap(),
            birth_province: CFString::new("PD".into()).unwrap(),
        };

        assert_eq!(
            CodiceFiscale::try_from(&subject).unwrap().get(),
            "GLNGCR56P10G224Q"
        );
    }
}
