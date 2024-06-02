use std::ops::Deref;

#[derive(Debug, PartialEq, Eq)]
pub enum CFStringError {
    Empty,
    NonAscii,
    NonAlphabetic,
}

impl std::error::Error for CFStringError {}
impl std::fmt::Display for CFStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            CFStringError::Empty => "string must not be empty",
            CFStringError::NonAscii => "string must be valid ascii",
            CFStringError::NonAlphabetic => "string must only have alphabetic or space characters",
        };
        write!(f, "{message}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CFString<S>(S);

impl<S: AsRef<str>> CFString<S> {
    pub fn new(s: S) -> Result<Self, CFStringError> {
        if s.as_ref().is_empty() {
            return Err(CFStringError::Empty);
        }
        Self::verify_ascii_alphabetical_or_space(s.as_ref())?;

        Ok(Self(s))
    }

    fn verify_ascii_alphabetical_or_space(s: &str) -> Result<(), CFStringError> {
        for b in s.as_bytes() {
            if *b == b' ' {
                continue;
            }
            if !b.is_ascii() {
                return Err(CFStringError::NonAscii);
            }
            if !b.is_ascii_alphabetic() {
                return Err(CFStringError::NonAlphabetic);
            }
        }

        Ok(())
    }

    pub fn as_deref(&self) -> CFString<&S::Target>
    where
        S: Deref,
    {
        CFString(&*self.0)
    }
}

impl<S> Deref for CFString<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfstring_short() {
        assert_eq!(CFString::new("W"), Ok(CFString("W")));
    }

    #[test]
    fn test_cfstring_empty() {
        assert_eq!(CFString::new(""), Err(CFStringError::Empty));
    }

    #[test]
    fn test_cfstring_accents() {
        let err = Err(CFStringError::NonAscii);

        assert_eq!(CFString::new("à"), err);
        assert_eq!(CFString::new("á"), err);
        assert_eq!(CFString::new("è"), err);
        assert_eq!(CFString::new("é"), err);
        assert_eq!(CFString::new("ì"), err);
        assert_eq!(CFString::new("í"), err);
        assert_eq!(CFString::new("ò"), err);
        assert_eq!(CFString::new("ó"), err);
        assert_eq!(CFString::new("ù"), err);
        assert_eq!(CFString::new("ú"), err);
    }
}
