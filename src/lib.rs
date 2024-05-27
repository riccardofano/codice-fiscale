enum Gender {
    Male,
    Female,
}

struct Subject {
    first_name: String,
    last_name: String,
    birth_date: String,
    gender: Gender,
    birth_place: String,
    birth_province: String,
}

struct CodiceFiscale(String);

impl CodiceFiscale {
    fn parse() -> Result<Self, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
