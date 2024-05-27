const VOWELS: [char; 6] = ['A', 'E', 'I', 'O', 'U', ' '];
const CONSONANTS: [char; 22] = [
    'B', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X',
    'Y', 'Z', ' ',
];

#[derive(Default)]
enum Gender {
    #[default]
    Male,
    Female,
}

#[derive(Default)]
struct Subject {
    first_name: String,
    last_name: String,
    birth_date: String,
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
}

struct CodiceFiscale(String);

impl CodiceFiscale {
    fn parse() -> Result<Self, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_name_code_enough_consonants() {
        let sub = Subject {
            last_name: "Rossi".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "RSS");
    }

    #[test]
    fn last_name_code_vowels_needed() {
        let sub = Subject {
            last_name: "Bigi".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "BGI");
    }

    #[test]
    fn last_name_code_space_inside() {
        let sub = Subject {
            last_name: "De Rossi".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "DRS");
    }

    #[test]
    fn last_name_code_short() {
        let sub = Subject {
            last_name: "Yu".into(),
            ..Default::default()
        };

        assert_eq!(&sub.last_name_code(), "YUX");
    }

    #[test]
    fn first_name_consonants() {
        let sub = Subject {
            first_name: "Massimo".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "MSM");
    }

    #[test]
    fn first_name_vowels_needed() {
        let sub = Subject {
            first_name: "Mario".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "MRA");
    }

    #[test]
    fn first_name_space_inside() {
        let sub = Subject {
            first_name: "Maria Teresa".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "MTR");
    }

    #[test]
    fn first_name_short() {
        let sub = Subject {
            first_name: "Li".into(),
            ..Default::default()
        };

        assert_eq!(&sub.first_name_code(), "LIX");
    }
}
