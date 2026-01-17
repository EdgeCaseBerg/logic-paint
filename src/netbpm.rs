use std::fs::read_to_string;

#[derive(Debug)]
pub struct Pbm {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<bool>,
}

type PbmResult<T> = Result<T, LoadPbmErr>;

#[derive(Debug)]
pub enum LoadPbmErr {
    MissingHeader,
    InvalidHeader { found: String },
    MissingSizeError,
    InvalidSizeError { found: String },
}

impl Pbm {
    pub fn from_ascii(string: &str) -> PbmResult<Pbm> {
        let mut characters = string.split_whitespace();
        let header = characters.next().ok_or(LoadPbmErr::MissingHeader)?;
        let "P1" = header else {
            return Err(LoadPbmErr::InvalidHeader {
                found: header.to_owned(),
            });
        };

        let width = characters.next();
        let height = characters.next();

        if width.is_none() || height.is_none() {
            return Err(LoadPbmErr::MissingSizeError);
        }
        let (width, height): (u16, u16) = match (width.unwrap().parse(), height.unwrap().parse()) {
            (Ok(w), Ok(h)) => (w, h),
            (res1, res2) => {
                let error_string = format!("{:?}, {:?}", res1, res2);
                return Err(LoadPbmErr::InvalidSizeError {
                    found: "Failed to convert size line into u16".to_owned() + &error_string,
                });
            }
        };

        let cells = characters
            .filter_map(|c| match c {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            })
            .collect();

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

    #[rustfmt::skip]
    #[test]
    fn can_load_sample_ascii() {
        let data =
            read_to_string("assets/P1.pbm").expect("Could not load asset file for test (P1.pbm)");

        if let Ok(pbm) = Pbm::from_ascii(&data) {
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
            panic!("Failed to load PBM file, got {:?}", Pbm::from_ascii(&data));
        }
    }
}
