use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Row {
    pub start: u32,
    pub end: u32,
    pub method: &'static str,
    pub weights: [i16; 14],
    pub exception: Option<&'static str>,
}

#[derive(Debug, PartialEq)]
pub enum TableError {
    FieldCount(usize),
    SortCode(String),
    StartAfterEnd { start: u32, end: u32 },
    UnknownMethod(String),
    Weight(String),
    UnknownException(String),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldCount(n) => write!(f, "expected 17 or 18 fields, found {n}"),
            Self::SortCode(s) => write!(f, "invalid sort code {s:?}"),
            Self::StartAfterEnd { start, end } => {
                write!(f, "range start {start:06} is after end {end:06}")
            }
            Self::UnknownMethod(s) => write!(f, "unknown modulus method {s:?}"),
            Self::Weight(s) => write!(f, "invalid weight {s:?}"),
            Self::UnknownException(s) => write!(f, "unknown exception {s:?}"),
        }
    }
}

pub fn parse_row(line: &str) -> Result<Row, TableError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() != 17 && fields.len() != 18 {
        return Err(TableError::FieldCount(fields.len()));
    }

    let start = sort_code(fields[0])?;
    let end = sort_code(fields[1])?;
    if start > end {
        return Err(TableError::StartAfterEnd { start, end });
    }

    let method = method(fields[2])?;

    let mut weights = [0i16; 14];
    for (weight, text) in weights.iter_mut().zip(&fields[3..17]) {
        *weight = text
            .parse()
            .map_err(|_| TableError::Weight((*text).to_owned()))?;
    }

    let exception = match fields.get(17) {
        Some(text) => Some(exception(text)?),
        None => None,
    };

    Ok(Row {
        start,
        end,
        method,
        weights,
        exception,
    })
}

fn sort_code(text: &str) -> Result<u32, TableError> {
    if text.len() != 6 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(TableError::SortCode(text.to_owned()));
    }

    text.parse()
        .map_err(|_| TableError::SortCode(text.to_owned()))
}

fn method(text: &str) -> Result<&'static str, TableError> {
    match text {
        "MOD10" => Ok("Method::Mod10"),
        "MOD11" => Ok("Method::Mod11"),
        "DBLAL" => Ok("Method::DblAl"),
        other => Err(TableError::UnknownMethod(other.to_owned())),
    }
}

macro_rules! generate_exceptions_match {
    ($e:expr, [$($l:literal),* $(,)?], $i:expr) => {
        match $e {
            $(
                $l => Ok(concat!("Exception::E", $l)),
            )*
            _ => Err($i()),
        }
    };
}

fn exception(text: &str) -> Result<&'static str, TableError> {
    let unknown_exception = || TableError::UnknownException(text.to_owned());
    let code: u8 = text.parse().map_err(|_| unknown_exception())?;

    generate_exceptions_match!(
        code,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,],
        unknown_exception
    )
}

impl Row {
    pub fn to_rust(&self) -> String {
        let exception = match self.exception {
            Some(e) => format!("Some({e})"),
            None => "None".to_owned(),
        };

        format!(
            "    WeightRow {{ start: {}, end: {}, method: {}, weights: {:?}, exception: {} }},",
            sort_code_rust(self.start),
            sort_code_rust(self.end),
            self.method,
            self.weights,
            exception
        )
    }
}

pub fn parse_substitution(line: &str) -> Result<(u32, u32), TableError> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() != 2 {
        return Err(TableError::FieldCount(fields.len()));
    }
    Ok((sort_code(fields[0])?, sort_code(fields[1])?))
}

pub fn sort_code_rust(value: u32) -> String {
    let digits: Vec<String> = format!("{value:06}")
        .bytes()
        .map(|b| (b - b'0').to_string())
        .collect();
    format!("SortCode::from_digits([{}])", digits.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_row_without_exception() {
        let row = parse_row("010004 016715 MOD11 0 0 0 0 0 0 8 7 6 5 4 3 2 1").unwrap();
        assert_eq!(row.start, 10004);
        assert_eq!(row.end, 16715);
        assert_eq!(row.method, "Method::Mod11");
        assert_eq!(row.weights, [0, 0, 0, 0, 0, 0, 8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(row.exception, None);
    }

    #[test]
    fn parses_negative_weight_and_exception() {
        let row = parse_row("070116 070116 MOD11 0 0 7 6 5 8 9 4 5 6 7 8 9 -1 12").unwrap();
        assert_eq!(row.weights[13], -1);
        assert_eq!(row.exception, Some("Exception::E12"));
    }

    #[test]
    fn handles_crlf_and_extra_spaces() {
        assert!(parse_row("040003   040003 DBLAL 2 1 2 1 2 1 8 7 6 5 4 3 2 1\r").is_ok());
    }

    #[test]
    fn sort_code_rust_keeps_leading_zeros() {
        assert_eq!(
            sort_code_rust(10004),
            "SortCode::from_digits([0, 1, 0, 0, 0, 4])"
        );
        assert_eq!(
            sort_code_rust(938173),
            "SortCode::from_digits([9, 3, 8, 1, 7, 3])"
        );
    }

    #[test]
    fn rejects_bad_input() {
        let base = "040003 040003";
        let w = "2 1 2 1 2 1 8 7 6 5 4 3 2 1";
        assert_eq!(
            parse_row(&format!("{base} MOD12 {w}")),
            Err(TableError::UnknownMethod("MOD12".into()))
        );
        assert_eq!(
            parse_row(&format!("{base} MOD10 {w} 15")),
            Err(TableError::UnknownException("15".into()))
        );
        assert_eq!(
            parse_row("040003 040003 MOD10 1 2 3"),
            Err(TableError::FieldCount(6))
        );
        assert_eq!(
            parse_row(&format!("04003 040003 MOD10 {w}")),
            Err(TableError::SortCode("04003".into()))
        );
        assert_eq!(
            parse_row(&format!("040004 040003 MOD10 {w}")),
            Err(TableError::StartAfterEnd {
                start: 40004,
                end: 40003
            })
        );
    }
}
