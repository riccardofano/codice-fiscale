use std::error::Error;
use std::sync::OnceLock;

use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Utc;

use crate::{all_subsets, CFString, DecodedData, Gender, Subject};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

const VOWELS: [char; 6] = ['A', 'E', 'I', 'O', 'U', ' '];
const CONSONANTS: [char; 22] = [
    'B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X',
    'Y', 'Z', ' ',
];
const MONTH_CODES: [char; 12] = ['A', 'B', 'C', 'D', 'E', 'H', 'L', 'M', 'P', 'R', 'S', 'T'];
const CHECK_CODE_NUM_ODD: [usize; 10] = [1, 0, 5, 7, 9, 13, 15, 17, 19, 21];
const CHECK_CODE_NUM_EVEN: [usize; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
const CHECK_CODE_LET_ODD: [usize; 26] = [
    1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6, 8, 12, 14, 16, 10, 22, 25, 24, 23,
];
const CHECK_CODE_LET_EVEN: [usize; 26] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
];
const OMOCODE_POSITIONS: [usize; 7] = [6, 7, 9, 10, 12, 13, 14];
const OMOCODE_LETTERS: [char; 10] = ['L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V'];
static OMOCODE_SUBSETS: OnceLock<Vec<Vec<usize>>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct CodiceFiscale(String);

impl CodiceFiscale {
    pub fn encode(subject: &Subject) -> Result<Self, GenerationError> {
        Self::try_from(subject)
    }

    pub fn decode(&self) -> Result<DecodedData, ValidationError> {
        let code = self.normalize()?;
        let code = code.get();

        let (birth_date, gender) = Self::decode_date(code)?;
        let (birth_place, birth_province) =
            Self::decode_birth_place(code).ok_or(ValidationError::UnknownPlaceCode)?;

        Ok(DecodedData {
            birth_date,
            gender,
            birth_place,
            birth_province,
        })
    }

    pub fn normalize(&self) -> Result<Self, ValidationError> {
        let mut bytes = self.0.as_bytes()[0..15].to_vec();
        for position in OMOCODE_POSITIONS {
            let current = bytes[position] as char;
            if current.is_ascii_digit() {
                continue;
            }

            let letter_index = OMOCODE_LETTERS
                .iter()
                .position(|&l| l == current)
                .ok_or(ValidationError::InvalidOmocodeLetter)?;
            bytes[position] = letter_index as u8 + b'0';
        }

        let mut string = String::from_utf8(bytes).unwrap();
        let checksum =
            Self::compute_checksum(&string).map_err(|_| ValidationError::IncorrectLength(15))?;
        string.push(checksum);

        Ok(CodiceFiscale(string))
    }

    pub fn get(&self) -> &str {
        &self.0
    }

    pub fn encode_last_name(last_name: CFString<&str>) -> String {
        let consonants = last_name.to_ascii_uppercase().replace(VOWELS, "");
        let vowels = last_name.to_ascii_uppercase().replace(CONSONANTS, "");

        format!("{consonants}{vowels}XXX")[..3].to_owned()
    }

    pub fn encode_first_name(first_name: CFString<&str>) -> String {
        let consonants = first_name.to_ascii_uppercase().replace(VOWELS, "");
        let b = consonants.as_bytes();

        if b.len() > 3 {
            format!("{}{}{}", b[0] as char, b[2] as char, b[3] as char)
        } else {
            let vowels = first_name.to_ascii_uppercase().replace(CONSONANTS, "");
            format!("{consonants}{vowels}XXX")[..3].to_owned()
        }
    }

    pub fn encode_birth_date(birth_date: NaiveDate, gender: Gender) -> String {
        let year = birth_date.year();
        let month = MONTH_CODES[birth_date.month0() as usize];
        let mut day = birth_date.day();

        if gender == Gender::Female {
            day += 40;
        }

        format!("{year:02}{month}{day:02}", year = year % 100)
    }

    pub fn encode_birth_place(city: CFString<&str>, province: CFString<&str>) -> Option<String> {
        let municipality = city.replace(' ', "-").to_ascii_lowercase();
        let province = province.to_ascii_uppercase();

        let key = format!("{municipality},{province}");

        if let Some(active_found) = ACTIVE_PLACES.get(&key) {
            return Some(active_found.to_string());
        }

        INACTIVE_PLACES.get(&key).map(|p| p.to_string())
    }

    pub fn compute_checksum(partial_cf: &str) -> Result<char, GenerationError> {
        if partial_cf.len() != 15 {
            return Err(GenerationError::IncorrectChecksumInputLength);
        }

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
        Ok((sum as u8 + b'A') as char)
    }

    pub fn is_omocode(&self) -> bool {
        let cf = self.0.as_bytes();
        OMOCODE_POSITIONS
            .iter()
            .any(|&pos| !cf[pos].is_ascii_digit())
    }

    pub fn all_omocodes(&self) -> Vec<CodiceFiscale> {
        let cf = &self.0.as_bytes()[0..15];
        let subsets = OMOCODE_SUBSETS.get_or_init(|| all_subsets(&OMOCODE_POSITIONS));

        let mut all_cfs = Vec::new();
        for subset in subsets {
            let mut code = cf.to_vec();
            for &position in subset {
                let digit = code[position] - b'0';
                code[position] = OMOCODE_LETTERS[digit as usize] as u8;
            }
            let code_as_str = std::str::from_utf8(&code).unwrap();
            let checksum = Self::compute_checksum(code_as_str).unwrap();
            all_cfs.push(CodiceFiscale(format!("{code_as_str}{checksum}")));
        }

        all_cfs
    }

    pub fn decode_date(cf: &str) -> Result<(NaiveDate, Gender), ValidationError> {
        let bytes = &cf.as_bytes()[6..11];

        let mut year = ((bytes[0] - b'0') * 10 + (bytes[1] - b'0')) as i32;
        let month = bytes[2] as char;
        let mut day = (bytes[3] - b'0') * 10 + (bytes[4] - b'0');

        let month = MONTH_CODES
            .iter()
            .position(|&c| c == month)
            .ok_or(ValidationError::InvalidMonthLetter)?;
        let gender = if day > 40 {
            day -= 40;
            Gender::Female
        } else {
            Gender::Male
        };

        let curr_year = Utc::now().year();
        year += if curr_year % 100 > year { 2000 } else { 1900 };

        let date = NaiveDate::from_ymd_opt(year, (month + 1) as u32, day as u32)
            .ok_or(ValidationError::InvalidDate)?;

        Ok((date, gender))
    }

    pub fn decode_birth_place(cf: &str) -> Option<(String, String)> {
        let bytes = &cf.as_bytes()[11..15];
        let code = std::str::from_utf8(bytes).unwrap();

        if let Some((&key, _)) = ACTIVE_PLACES.into_iter().find(|(_, &v)| v == code) {
            return Some(Self::place_to_string(key));
        };

        INACTIVE_PLACES
            .into_iter()
            .find(|(_, &v)| v == code)
            .map(|(&key, _)| Self::place_to_string(key))
    }

    fn place_to_string(place_key: &str) -> (String, String) {
        let (city, province) = place_key.split_once(',').unwrap();
        (city.replace('-', " "), province.to_owned())
    }
}

impl std::str::FromStr for CodiceFiscale {
    type Err = ValidationError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.len() != 16 {
            return Err(ValidationError::IncorrectLength(16));
        }
        if string.chars().any(|c| !c.is_alphanumeric()) {
            return Err(ValidationError::NonAlphanumeric);
        }

        Ok(Self(string.to_owned()))
    }
}

impl TryFrom<&Subject> for CodiceFiscale {
    type Error = GenerationError;

    fn try_from(value: &Subject) -> Result<Self, Self::Error> {
        let mut output = String::with_capacity(16);

        output.push_str(&Self::encode_last_name(value.last_name.as_deref()));
        output.push_str(&Self::encode_first_name(value.first_name.as_deref()));
        output.push_str(&Self::encode_birth_date(value.birth_date, value.gender));

        let place_code = Self::encode_birth_place(
            value.birth_place.as_deref(),
            value.birth_province.as_deref(),
        )
        .ok_or(Self::Error::BelfioreCodeNotFound)?;
        output.push_str(&place_code);
        output.push(Self::compute_checksum(&output)?);

        Ok(Self(output))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GenerationError {
    BelfioreCodeNotFound,
    IncorrectChecksumInputLength,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    IncorrectLength(usize),
    InvalidDate,
    InvalidMonthLetter,
    InvalidOmocodeLetter,
    NonAlphanumeric,
    UnknownPlaceCode,
}

impl Error for GenerationError {}
impl Error for ValidationError {}
impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::BelfioreCodeNotFound => "could not find belfiore code for this city and province",
            Self::IncorrectChecksumInputLength => "checksum input must be 15 characters long",
        };
        write!(f, "{message}")
    }
}
impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidDate => "the codice fiscale does not contain a valid date",
            Self::IncorrectLength(len) => return write!(f, "input must be {len} characters long"),
            Self::InvalidOmocodeLetter => "codice fiscale contains invalid omocode letter",
            Self::InvalidMonthLetter => "the letter used for the month is invalid",
            Self::NonAlphanumeric => "characters must be alphabetical letters or numbers",
            Self::UnknownPlaceCode => "could not find place associated with code in codice fiscale",
        };
        write!(f, "{message}")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_last_name_code_enough_consonants() {
        let name = CFString::new("Rossi").unwrap();
        assert_eq!(&CodiceFiscale::encode_last_name(name), "RSS");
    }

    #[test]
    fn test_last_name_code_vowels_needed() {
        let name = CFString::new("Bigi").unwrap();
        assert_eq!(&CodiceFiscale::encode_last_name(name), "BGI");
    }

    #[test]
    fn test_last_name_code_space_inside() {
        let name = CFString::new("De Rossi").unwrap();
        assert_eq!(&CodiceFiscale::encode_last_name(name), "DRS");
    }

    #[test]
    fn test_last_name_code_short() {
        let name = CFString::new("Yu").unwrap();
        assert_eq!(&CodiceFiscale::encode_last_name(name), "YUX");
    }

    #[test]
    fn test_first_name_consonants() {
        let name = CFString::new("Massimo").unwrap();
        assert_eq!(&CodiceFiscale::encode_first_name(name), "MSM");
    }

    #[test]
    fn test_first_name_vowels_needed() {
        let name = CFString::new("Mario").unwrap();
        assert_eq!(&CodiceFiscale::encode_first_name(name), "MRA");
    }

    #[test]
    fn test_first_name_space_inside() {
        let name = CFString::new("Maria Teresa").unwrap();
        assert_eq!(&CodiceFiscale::encode_first_name(name), "MTR");
    }

    #[test]
    fn test_first_name_short() {
        let name = CFString::new("Li").unwrap();
        assert_eq!(&CodiceFiscale::encode_first_name(name), "LIX");
    }

    #[test]
    fn test_first_name_super_short() {
        let name = CFString::new("W").unwrap();
        assert_eq!(&CodiceFiscale::encode_first_name(name), "WXX");
    }

    #[test]
    fn test_birth_date_code() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let res = CodiceFiscale::encode_birth_date(birth_date, Gender::Male);
        assert_eq!(&res, "24T31");
    }

    #[test]
    fn test_birth_date_small_day() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();

        let res = CodiceFiscale::encode_birth_date(birth_date, Gender::Male);
        assert_eq!(&res, "24T05");
    }

    #[test]
    fn test_birth_date_female() {
        let birth_date = NaiveDate::from_ymd_opt(2024, 12, 5).unwrap();

        let res = CodiceFiscale::encode_birth_date(birth_date, Gender::Female);
        assert_eq!(&res, "24T45");
    }

    #[test]
    fn test_birth_date_ends_with_0_something() {
        let birth_date = NaiveDate::from_ymd_opt(2003, 12, 6).unwrap();
        let res = CodiceFiscale::encode_birth_date(birth_date, Gender::Male);
        assert_eq!(&res, "03T06");
    }

    #[test]
    fn test_birth_place() {
        let place = CFString::new("Abano").unwrap();
        let province = CFString::new("PD").unwrap();
        let res = CodiceFiscale::encode_birth_place(place, province);
        assert_eq!(res.as_deref(), Some("A001"));
    }

    #[test]
    fn test_birth_place_not_found() {
        let place = CFString::new("I dont exist").unwrap();
        let province = CFString::new("PD").unwrap();
        let res = CodiceFiscale::encode_birth_place(place, province);
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

    #[test]
    fn test_all_omocodes() {
        // From https://github.com/fabiocaccamo/python-codicefiscale/blob/main/tests/test_codicefiscale.py#L598
        let mut expected = vec![
            "CCCFBA85D03L21VE",
            "CCCFBA85D03L2M9A",
            "CCCFBA85D03LN19E",
            "CCCFBA85D0PL219L",
            "CCCFBA85DL3L219A",
            "CCCFBA8RD03L219B",
            "CCCFBAU5D03L219M",
            "CCCFBA85D03L2MVP",
            "CCCFBA85D03LN1VT",
            "CCCFBA85D0PL21VA",
            "CCCFBA85DL3L21VP",
            "CCCFBA8RD03L21VQ",
            "CCCFBAU5D03L21VB",
            "CCCFBA85D03LNM9P",
            "CCCFBA85D0PL2M9W",
            "CCCFBA85DL3L2M9L",
            "CCCFBA8RD03L2M9M",
            "CCCFBAU5D03L2M9X",
            "CCCFBA85D0PLN19A",
            "CCCFBA85DL3LN19P",
            "CCCFBA8RD03LN19Q",
            "CCCFBAU5D03LN19B",
            "CCCFBA85DLPL219W",
            "CCCFBA8RD0PL219X",
            "CCCFBAU5D0PL219I",
            "CCCFBA8RDL3L219M",
            "CCCFBAU5DL3L219X",
            "CCCFBAURD03L219Y",
            "CCCFBA85D03LNMVE",
            "CCCFBA85D0PL2MVL",
            "CCCFBA85DL3L2MVA",
            "CCCFBA8RD03L2MVB",
            "CCCFBAU5D03L2MVM",
            "CCCFBA85D0PLN1VP",
            "CCCFBA85DL3LN1VE",
            "CCCFBA8RD03LN1VF",
            "CCCFBAU5D03LN1VQ",
            "CCCFBA85DLPL21VL",
            "CCCFBA8RD0PL21VM",
            "CCCFBAU5D0PL21VX",
            "CCCFBA8RDL3L21VB",
            "CCCFBAU5DL3L21VM",
            "CCCFBAURD03L21VN",
            "CCCFBA85D0PLNM9L",
            "CCCFBA85DL3LNM9A",
            "CCCFBA8RD03LNM9B",
            "CCCFBAU5D03LNM9M",
            "CCCFBA85DLPL2M9H",
            "CCCFBA8RD0PL2M9I",
            "CCCFBAU5D0PL2M9T",
            "CCCFBA8RDL3L2M9X",
            "CCCFBAU5DL3L2M9I",
            "CCCFBAURD03L2M9J",
            "CCCFBA85DLPLN19L",
            "CCCFBA8RD0PLN19M",
            "CCCFBAU5D0PLN19X",
            "CCCFBA8RDL3LN19B",
            "CCCFBAU5DL3LN19M",
            "CCCFBAURD03LN19N",
            "CCCFBA8RDLPL219I",
            "CCCFBAU5DLPL219T",
            "CCCFBAURD0PL219U",
            "CCCFBAURDL3L219J",
            "CCCFBA85D0PLNMVA",
            "CCCFBA85DL3LNMVP",
            "CCCFBA8RD03LNMVQ",
            "CCCFBAU5D03LNMVB",
            "CCCFBA85DLPL2MVW",
            "CCCFBA8RD0PL2MVX",
            "CCCFBAU5D0PL2MVI",
            "CCCFBA8RDL3L2MVM",
            "CCCFBAU5DL3L2MVX",
            "CCCFBAURD03L2MVY",
            "CCCFBA85DLPLN1VA",
            "CCCFBA8RD0PLN1VB",
            "CCCFBAU5D0PLN1VM",
            "CCCFBA8RDL3LN1VQ",
            "CCCFBAU5DL3LN1VB",
            "CCCFBAURD03LN1VC",
            "CCCFBA8RDLPL21VX",
            "CCCFBAU5DLPL21VI",
            "CCCFBAURD0PL21VJ",
            "CCCFBAURDL3L21VY",
            "CCCFBA85DLPLNM9W",
            "CCCFBA8RD0PLNM9X",
            "CCCFBAU5D0PLNM9I",
            "CCCFBA8RDL3LNM9M",
            "CCCFBAU5DL3LNM9X",
            "CCCFBAURD03LNM9Y",
            "CCCFBA8RDLPL2M9T",
            "CCCFBAU5DLPL2M9E",
            "CCCFBAURD0PL2M9F",
            "CCCFBAURDL3L2M9U",
            "CCCFBA8RDLPLN19X",
            "CCCFBAU5DLPLN19I",
            "CCCFBAURD0PLN19J",
            "CCCFBAURDL3LN19Y",
            "CCCFBAURDLPL219F",
            "CCCFBA85DLPLNMVL",
            "CCCFBA8RD0PLNMVM",
            "CCCFBAU5D0PLNMVX",
            "CCCFBA8RDL3LNMVB",
            "CCCFBAU5DL3LNMVM",
            "CCCFBAURD03LNMVN",
            "CCCFBA8RDLPL2MVI",
            "CCCFBAU5DLPL2MVT",
            "CCCFBAURD0PL2MVU",
            "CCCFBAURDL3L2MVJ",
            "CCCFBA8RDLPLN1VM",
            "CCCFBAU5DLPLN1VX",
            "CCCFBAURD0PLN1VY",
            "CCCFBAURDL3LN1VN",
            "CCCFBAURDLPL21VU",
            "CCCFBA8RDLPLNM9I",
            "CCCFBAU5DLPLNM9T",
            "CCCFBAURD0PLNM9U",
            "CCCFBAURDL3LNM9J",
            "CCCFBAURDLPL2M9Q",
            "CCCFBAURDLPLN19U",
            "CCCFBA8RDLPLNMVX",
            "CCCFBAU5DLPLNMVI",
            "CCCFBAURD0PLNMVJ",
            "CCCFBAURDL3LNMVY",
            "CCCFBAURDLPL2MVF",
            "CCCFBAURDLPLN1VJ",
            "CCCFBAURDLPLNM9F",
            "CCCFBAURDLPLNMVU",
        ];
        expected.sort();

        let all_omocodes = CodiceFiscale("CCCFBA85D03L219P".into()).all_omocodes();
        let mut all_strs = all_omocodes
            .iter()
            .map(|cf| cf.0.as_str())
            .collect::<Vec<_>>();
        all_strs.sort();

        assert_eq!(all_strs, expected);
    }

    #[test]
    fn test_normalize_from_omocode() {
        let expected = "CCCFBA85D03L219P";

        let cf = CodiceFiscale("CCCFBA85DLPLNM9W".into());
        assert_eq!(cf.normalize().unwrap().get(), expected);

        let cf = CodiceFiscale("CCCFBAURDLPLNMVU".into());
        assert_eq!(cf.normalize().unwrap().get(), expected);

        let cf = CodiceFiscale("CCCFBA85D03LN19E".into());
        assert_eq!(cf.normalize().unwrap().get(), expected);
    }

    #[test]
    fn test_decode_date() {
        let expected_date = NaiveDate::from_ymd_opt(1985, 4, 3).unwrap();
        let expected_gender = Gender::Male;
        assert_eq!(
            CodiceFiscale::decode_date("CCCFBA85D03L219P").unwrap(),
            (expected_date, expected_gender)
        );
    }

    #[test]
    fn test_decode_active_place() {
        let expected_city = "pedivigliano".to_owned();
        let expected_province = "CS".to_owned();

        let res = CodiceFiscale::decode_birth_place("CCCFBA85D03G411P");
        assert_eq!(res, Some((expected_city, expected_province)));
    }

    #[test]
    fn test_decode_inactive_place() {
        let expected_city = "carano".to_owned();
        let expected_province = "TN".to_owned();

        let res = CodiceFiscale::decode_birth_place("CCCFBA85D03B723P");
        assert_eq!(res, Some((expected_city, expected_province)));
    }

    #[test]
    fn test_decode_unknown_place() {
        let res = CodiceFiscale::decode_birth_place("CCCFBA85D03C008P");
        assert_eq!(res, None);
    }

    #[test]
    fn test_decode_multiple_word_city() {
        let expected_city = "abbadia lariana".to_owned();
        let expected_province = "LC".to_owned();

        let res = CodiceFiscale::decode_birth_place("CCCFBA85D03A005R");
        assert_eq!(res, Some((expected_city, expected_province)));
    }

    #[test]
    fn test_decode_data() {
        let code = CodiceFiscale::from_str("GLNGCR56P10G224Q").unwrap();

        let expected = DecodedData {
            birth_date: NaiveDate::from_ymd_opt(1956, 9, 10).unwrap(),
            gender: Gender::Male,
            birth_place: "padova".into(),
            birth_province: "PD".into(),
        };

        assert_eq!(code.decode().unwrap(), expected);
    }
}
