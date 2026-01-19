use std::str::FromStr;

#[derive(Debug)]
pub struct Pbm {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<bool>,
}

impl Pbm {
    pub fn rows(&self) -> Vec<Vec<bool>> {
        let mut result = vec![];
        for chunk in self.cells.chunks(self.width) {
            result.push(chunk.to_vec());
        }
        result
    }

    pub fn cols(&self) -> Vec<Vec<bool>> {
        let mut cols = vec![vec![]; self.width];
        for c in 0..self.width {
            for row in self.rows() {
                cols[c].push(row[c]);
            }
        }
        cols
    }
}

pub type PbmResult<T> = Result<T, LoadPbmErr>;

#[derive(Debug)]
pub enum LoadPbmErr {
    MissingHeader,
    InvalidHeader { found: String },
    MissingWidthError,
    MissingHeightError,
    InvalidWidthError { found: String, reason: String },
    InvalidHeightError { found: String, reason: String },
    InvalidMatrixSize { expected: usize, got: usize },
    UnexpectedCellValue { found: String },
}

impl std::error::Error for LoadPbmErr {}

impl std::fmt::Display for LoadPbmErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use LoadPbmErr::*;
        let s = match self {
            MissingHeader => "missing expected header for pbm file (should be P1)".to_owned(),
            InvalidHeader { found } => format!("invalid header found: {}", found),
            MissingWidthError => "missing width in pbm file".to_owned(),
            MissingHeightError => "missing height in pbm file".to_owned(),
            InvalidWidthError { found, reason } => {
                "invalid width of ".to_owned() + found + ": " + reason
            }
            InvalidHeightError { found, reason } => {
                "invalid width of ".to_owned() + found + ": " + reason
            }
            InvalidMatrixSize { expected, got } => {
                format!("invalid matrix cell, expected: {} got {}", expected, got)
            }
            UnexpectedCellValue { found } => format!("invalid pbm cell value: {}", found),
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Pbm {
    type Err = LoadPbmErr;
    fn from_str(string: &str) -> PbmResult<Pbm> {
        /* Ignore comment lines, but grab all the characters out otherwise. */
        let mut characters = string
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .flat_map(str::split_whitespace);

        let header = characters.next().ok_or(LoadPbmErr::MissingHeader)?;
        let "P1" = header else {
            return Err(LoadPbmErr::InvalidHeader {
                found: header.to_owned(),
            });
        };

        let width = characters.next().ok_or(LoadPbmErr::MissingWidthError)?;
        let width = width
            .parse::<usize>()
            .map_err(|e| LoadPbmErr::InvalidWidthError {
                found: width.to_owned(),
                reason: e.to_string(),
            })?;

        let height = characters.next().ok_or(LoadPbmErr::MissingHeightError)?;
        let height = height
            .parse::<usize>()
            .map_err(|e| LoadPbmErr::InvalidHeightError {
                found: height.to_owned(),
                reason: e.to_string(),
            })?;

        let cells: Vec<bool> = characters
            .map(|c| match c {
                "0" => Ok(false),
                "1" => Ok(true),
                _ => Err(LoadPbmErr::UnexpectedCellValue {
                    found: c.to_owned(),
                }),
            })
            .collect::<Result<_, _>>()?;

        let expected_count = width * height;
        if cells.len() != expected_count {
            return Err(LoadPbmErr::InvalidMatrixSize {
                expected: expected_count,
                got: cells.len(),
            });
        }

        Ok(Pbm {
            width,
            height,
            cells,
        })
    }
}

#[cfg(test)]
mod pbm_tests {
    use super::*;
    use std::fs::read_to_string;

    #[rustfmt::skip]
    #[test]
    fn can_load_sample_ascii() {
        let data =
            read_to_string("assets/P1.pbm").expect("Could not load asset file for test (P1.pbm)");

        let result: PbmResult<Pbm> = data.parse();
        if let Ok(pbm) = result {
            assert_eq!(pbm.width, 5);
            assert_eq!(pbm.height, 5);
            assert_eq!(pbm.cells, vec![
                false, true , true, true , false,
                false, true , true, true , false,
                false, false, true, false, false,
                false, true , true, true , false,
                false, true , true, true , false
            ]);
        } else {
            panic!("Failed to load PBM file, got {:?}", result);
        }
    }

    #[test]
    fn fails_to_load_bad_header() {
        let data = "P2\n1 1\n1";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::InvalidHeader { found }) => {
                assert_eq!(found, "P2");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_missing_width() {
        let data = "P1\n\n";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::MissingWidthError) => {}
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_missing_height() {
        let data = "P1\n1\n";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::MissingHeightError) => {}
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_width() {
        let data = "P1\nw 1\n";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::InvalidWidthError { found, reason }) => {
                assert_eq!(reason, "invalid digit found in string");
                assert_eq!(found, "w");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_height() {
        let data = "P1\n1 x\n";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::InvalidHeightError { found, reason }) => {
                assert_eq!(reason, "invalid digit found in string");
                assert_eq!(found, "x");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_matrix() {
        let data = "P1\n2 2\n1";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::InvalidMatrixSize { expected, got }) => {
                assert_eq!(expected, 4);
                assert_eq!(got, 1);
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_matrix_cell() {
        let data = "P1\n1 1\na";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::UnexpectedCellValue { found }) => {
                assert_eq!(found, "a");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    #[rustfmt::skip]
    fn returns_rows_as_expected() {
        let pbm = Pbm {
            width: 3,
            height: 4,
            cells: vec![
                false, false, false,
                true , true , true ,
                false, true , false,
                true,  false, false,
            ],
        };
        let mut rows = pbm.rows().into_iter();
        let [false, false, false] = rows.next().expect("bad iter 1st row")[..] else {
            eprintln!("{:?}", pbm.rows());
            panic!("failed 1st row")
        };
        let [true, true, true] = rows.next().expect("bad iter 2nd row")[..] else {
            eprintln!("{:?}", pbm.rows());
            panic!("failed 2nd row")
        };
        let [false, true, false] = rows.next().expect("bad iter 3rd row")[..] else {
            eprintln!("{:?}", pbm.rows());
            panic!("failed 3rd row")
        };
        let [true, false, false] = rows.next().expect("bad iter 4th row")[..] else {
            eprintln!("{:?}", pbm.rows());
            panic!("failed 4th row")
        };
        assert!(rows.next().is_none());
    }

    #[rustfmt::skip]
    #[test]
    fn returns_cols_as_expected() {
        let pbm = Pbm {
            width: 3,
            height: 4,
            cells: vec![
                false, false, false,
                true , true , true ,
                false, true , false,
                true,  false, false,
            ],
        };
        let mut cols = pbm.cols().into_iter();
        let [false, true, false, true] = cols.next().expect("bad iter 1st col")[..] else {
            eprintln!("{:?}", pbm.cols());
            panic!("failed 1st col")
        };
        let [false, true, true, false] = cols.next().expect("bad iter 2nd col")[..] else {
            eprintln!("{:?}", pbm.cols());
            panic!("failed 2nd col")
        };
        let [false, true, false, false] = cols.next().expect("bad iter 3rd col")[..] else {
            eprintln!("{:?}", pbm.cols());
            panic!("failed 3rd col")
        };
        assert!(cols.next().is_none());
    }
}
