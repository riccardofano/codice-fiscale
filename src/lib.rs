mod codice_fiscale;
mod string;

pub use chrono::NaiveDate;
pub use codice_fiscale::{CodiceFiscale, ACTIVE_PLACES, INACTIVE_PLACES};
pub use string::CFString;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Subject {
    pub first_name: CFString<String>,
    pub last_name: CFString<String>,
    pub birth_date: NaiveDate,
    pub gender: Gender,
    pub birth_place: CFString<String>,
    pub birth_province: CFString<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedData {
    pub birth_date: NaiveDate,
    pub gender: Gender,
    pub birth_place: String,
    pub birth_province: String,
}

/// Returns all subsets the elements of an array excepts the empty set
/// Which amounts to 2^n - 1 sets
fn all_subsets(array: &[usize]) -> Vec<Vec<usize>> {
    let mut subsets = Vec::new();
    if array.is_empty() {
        return subsets;
    }

    let last_index = array.len() - 1;
    let last_element = array[last_index];
    subsets.push(vec![last_element]);

    if last_index == 0 {
        return subsets;
    }

    let sub_array = &array[0..last_index];
    let sub_subsets = all_subsets(sub_array);

    for mut subset in sub_subsets {
        subsets.push(subset.clone());
        subset.push(last_element);
        subsets.push(subset);
    }

    subsets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_subsets() {
        let expected = vec![
            vec![3],
            vec![2],
            vec![2, 3],
            vec![1],
            vec![1, 3],
            vec![1, 2],
            vec![1, 2, 3],
        ];

        assert_eq!(all_subsets(&[1, 2, 3]), expected);
        // 2 to the power of 7 = 128, but that includes the empty case so 127
        assert_eq!(all_subsets(&[1, 2, 3, 4, 5, 6, 7]).len(), 127);
    }
}
