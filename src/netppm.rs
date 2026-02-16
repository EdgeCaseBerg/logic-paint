use std::str::FromStr;

#[derive(Debug)]
pub struct Ppm {
    pub width: usize,
    pub height: usize,
    pub max_value: u16,
    pub cells: Vec<[u16; 3]>,
}

impl Ppm {
    pub fn rows(&self) -> Vec<Vec<[u16; 3]>> {
        let mut result = vec![];
        for chunk in self.cells.chunks(self.width) {
            result.push(chunk.to_vec());
        }
        result
    }

    pub fn cols(&self) -> Vec<Vec<[u16; 3]>> {
        let mut cols = vec![vec![]; self.width];
        for c in 0..self.width {
            for row in self.rows() {
                cols[c].push(row[c]);
            }
        }
        cols
    }

    pub fn to_rgba(&self, cell: [u16; 3]) -> [f32; 4] {
        let max = self.max_value;
        let r = cell[0] as f32 / max as f32;
        let g = cell[1] as f32 / max as f32;
        let b = cell[2] as f32 / max as f32;
        [r, g, b, 1.0]
    }
}

pub type PpmResult<T> = Result<T, LoadPpmErr>;

#[derive(Debug)]
pub enum LoadPpmErr {
    MissingHeader,
    InvalidHeader { found: String },
    MissingWidthError,
    MissingHeightError,
    InvalidWidthError { found: String, reason: String },
    InvalidHeightError { found: String, reason: String },
    InvalidMatrixSize { expected: usize, got: usize },
    UnexpectedCellValue { found: String },
    MissingColorRangeError,
    InvalidColorRangeError { found: String, reason: String },
}

impl std::error::Error for LoadPpmErr {}

impl std::fmt::Display for LoadPpmErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use LoadPpmErr::*;
        let s = match self {
            MissingHeader => "missing expected header for ppm file (should be P3)".to_owned(),
            InvalidHeader { found } => format!("invalid header found: {}", found),
            MissingWidthError => "missing width in ppm file".to_owned(),
            MissingHeightError => "missing height in ppm file".to_owned(),
            InvalidWidthError { found, reason } => {
                "invalid width of ".to_owned() + found + ": " + reason
            }
            InvalidHeightError { found, reason } => {
                "invalid width of ".to_owned() + found + ": " + reason
            }
            InvalidMatrixSize { expected, got } => {
                format!("invalid matrix cell, expected: {} got {}", expected, got)
            }
            MissingColorRangeError => "missing color range in ppm file".to_owned(),
            UnexpectedCellValue { found } => format!("invalid ppm cell value: {}", found),
            InvalidColorRangeError { found, reason } => {
                "invalid color range of ".to_owned() + found + ": " + reason
            }
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Ppm {
    type Err = LoadPpmErr;
    fn from_str(string: &str) -> PpmResult<Ppm> {
        /* Ignore comment lines, but grab all the characters out otherwise. */
        let mut characters = string
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .flat_map(str::split_whitespace);

        let header = characters.next().ok_or(LoadPpmErr::MissingHeader)?;
        let "P3" = header else {
            return Err(LoadPpmErr::InvalidHeader {
                found: header.to_owned(),
            });
        };

        let width = characters.next().ok_or(LoadPpmErr::MissingWidthError)?;
        let width = width
            .parse::<usize>()
            .map_err(|e| LoadPpmErr::InvalidWidthError {
                found: width.to_owned(),
                reason: e.to_string(),
            })?;

        let height = characters.next().ok_or(LoadPpmErr::MissingHeightError)?;
        let height = height
            .parse::<usize>()
            .map_err(|e| LoadPpmErr::InvalidHeightError {
                found: height.to_owned(),
                reason: e.to_string(),
            })?;

        let max_value = characters
            .next()
            .ok_or(LoadPpmErr::MissingColorRangeError)?;
        let max_value =
            max_value
                .parse::<u16>()
                .map_err(|e| LoadPpmErr::InvalidColorRangeError {
                    found: max_value.to_owned(),
                    reason: e.to_string(),
                })?;

        let cells: Vec<u16> = characters
            .map(|c| match u16::from_str_radix(c, 10) {
                Ok(parsed_number) => {
                    if parsed_number > max_value {
                        Err(LoadPpmErr::InvalidColorRangeError {
                            found: c.to_string(),
                            reason: format!(
                                "parsed number was out of range defined by ppm file min:0 max:{}",
                                max_value
                            )
                            .to_string(),
                        })
                    } else {
                        Ok(parsed_number)
                    }
                }
                _ => Err(LoadPpmErr::UnexpectedCellValue {
                    found: c.to_owned(),
                }),
            })
            .collect::<Result<_, _>>()?;

        let expected_count = width * height;
        let (cells, []) = cells.as_chunks::<3>() else {
            return Err(LoadPpmErr::InvalidMatrixSize {
                expected: expected_count,
                got: cells.len(),
            });
        };
        let cells = cells.to_vec();

        Ok(Ppm {
            width,
            height,
            cells,
            max_value,
        })
    }
}

#[cfg(test)]
mod ppm_tests {
    use super::*;
    use std::fs::read_to_string;

    #[rustfmt::skip]
    #[test]
    fn can_load_sample_ascii() {
        let data =
            read_to_string("assets/sample.ppm").expect("Could not load asset file for test (sample.ppm)");

        let result: PpmResult<Ppm> = data.parse();
        if let Ok(ppm) = result {
            assert_eq!(ppm.width, 3);
            assert_eq!(ppm.height, 2);
            assert_eq!(ppm.cells, vec![
                [255,   0,   0],
                [0, 255,   0],
                [0,   0, 255],
                [255, 255,   0],
                [255, 255, 255],
                [0,   0,   0],
            ]);
        } else {
            panic!("Failed to load PPM file, got {:?}", result);
        }
    }

    #[test]
    fn fails_to_load_bad_header() {
        let data = "P2\n1 1\n1";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::InvalidHeader { found }) => {
                assert_eq!(found, "P2");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_missing_width() {
        let data = "P3\n\n";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::MissingWidthError) => {}
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_missing_height() {
        let data = "P3\n1\n";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::MissingHeightError) => {}
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_missing_color_range() {
        let data = "P3\n1\n1\n";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::MissingColorRangeError) => {}
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_width() {
        let data = "P3\nw 1\n";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::InvalidWidthError { found, reason }) => {
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
        let data = "P3\n1 x\n";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::InvalidHeightError { found, reason }) => {
                assert_eq!(reason, "invalid digit found in string");
                assert_eq!(found, "x");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_color_range() {
        let data = "P3\n1 1\nr";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::InvalidColorRangeError { found, reason }) => {
                assert_eq!(reason, "invalid digit found in string");
                assert_eq!(found, "r");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_matrix() {
        let data = "P3\n2 2\n1\n1";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::InvalidMatrixSize { expected, got }) => {
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
        let data = "P3\n1 1\n1\na";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::UnexpectedCellValue { found }) => {
                assert_eq!(found, "a");
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    fn fails_to_load_invalid_matrix_cell_oob() {
        let data = "P3\n1 1\n1\n3";
        let result: PpmResult<Ppm> = data.parse();
        match result {
            Err(LoadPpmErr::InvalidColorRangeError { found, reason }) => {
                assert_eq!(found, "3");
                assert_eq!(
                    reason,
                    "parsed number was out of range defined by ppm file min:0 max:1"
                );
            }
            weird => {
                panic!("Should not have parsed: {:?}", weird);
            }
        }
    }

    #[test]
    #[rustfmt::skip]
    fn returns_rows_as_expected() {
        let ppm = Ppm {
            width: 3,
            height: 2,
            max_value: 255,
            cells: vec![
                [255, 0, 0],
                [0, 255, 0],
                [0, 0, 255],
                [255, 255, 0],
                [255, 255, 255],
                [0, 0, 0],
            ],
        };
        let mut rows = ppm.rows().into_iter();
        let [
            [255, 0, 0], [0, 255, 0], [0, 0, 255]
        ] = rows.next().expect("bad iter 1st row")[..]
        else {
            eprintln!("{:?}", ppm.rows());
            panic!("failed 1st row")
        };
        let [
            [255, 255, 0], [255, 255, 255], [0, 0, 0]
        ] = rows.next().expect("bad iter 2nd row")[..]
        else {
            eprintln!("{:?}", ppm.rows());
            panic!("failed 2nd row")
        };
        assert!(rows.next().is_none());
    }

    #[test]
    #[rustfmt::skip]
    fn returns_cols_as_expected() {
        let ppm = Ppm {
            width: 3,
            height: 2,
            max_value: 255,
            cells: vec![
                [255, 0, 0],
                [0, 255, 0],
                [0, 0, 255],
                [255, 255, 0],
                [255, 255, 255],
                [0, 0, 0],
            ],
        };
        let mut cols = ppm.cols().into_iter();
        let [
            [255, 0, 0],
            [255, 255, 0]
        ] = cols.next().expect("bad iter 1st col")[..] else {
            eprintln!("{:?}", ppm.cols());
            panic!("failed 1st col")
        };
        let [
            [0, 255, 0],
            [255, 255, 255]
        ] = cols.next().expect("bad iter 2nd col")[..] else {
            eprintln!("{:?}", ppm.cols());
            panic!("failed 2nd col")
        };
        let [
            [0, 0, 255],
            [0, 0, 0]
        ] = cols.next().expect("bad iter 3rd col")[..] else {
            eprintln!("{:?}", ppm.cols());
            panic!("failed 3rd col")
        };
        assert!(cols.next().is_none());
    }

    #[test]
    fn to_rgba() {
        let ppm = Ppm {
            width: 3,
            height: 2,
            max_value: 255,
            cells: vec![
                [255, 0, 0],
                [0, 255, 0],
                [0, 0, 255],
                [255, 255, 0],
                [255, 255, 255],
                [0, 0, 0],
            ],
        };
        let rgba = ppm.to_rgba([255, 255, 255]);
        assert_eq!(rgba[0], 1.0);
        assert_eq!(rgba[1], 1.0);
        assert_eq!(rgba[2], 1.0);
        assert_eq!(rgba[3], 1.0);
    }
}
