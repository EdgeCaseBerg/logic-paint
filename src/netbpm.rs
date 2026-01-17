use std::str::FromStr;

#[derive(Debug)]
pub struct Pbm {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<bool>,
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
        let height = characters.next().ok_or(LoadPbmErr::MissingHeightError)?;

        let width = width
            .parse::<u16>()
            .map_err(|e| LoadPbmErr::InvalidWidthError {
                found: width.to_string(),
                reason: e.to_string(),
            })?;

        let height = height
            .parse::<u16>()
            .map_err(|e| LoadPbmErr::InvalidHeightError {
                found: height.to_string(),
                reason: e.to_string(),
            })?;

        let cells: Vec<bool> = characters
            .map(|c| match c {
                "0" => Ok(false),
                "1" => Ok(true),
                _ => Err(LoadPbmErr::UnexpectedCellValue {
                    found: c.to_string(),
                }),
            })
            .collect::<Result<_, _>>()?;

        let expected_count = width as usize * height as usize;
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
        let data = "P1\n100000 1\n";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::InvalidWidthError { found, reason } ) => {
                assert_eq!(reason, "number too large to fit in target type");
                assert_eq!(found, "100000");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_height() {
        let data = "P1\n1 100000\n";
        let result: PbmResult<Pbm> = data.parse();
        match result {
            Err(LoadPbmErr::InvalidHeightError { found, reason } ) => {
                assert_eq!(reason, "number too large to fit in target type");
                assert_eq!(found, "100000");
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
            Err(LoadPbmErr::InvalidMatrixSize { expected, got } ) => {
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
            Err(LoadPbmErr::UnexpectedCellValue { found, } ) => {
                assert_eq!(found, "a");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }    
}
