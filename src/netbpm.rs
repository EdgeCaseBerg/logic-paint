use std::fs::read_to_string;

#[derive(Debug)]
pub struct PBM {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<bool>,
}

type PbmResult<T> = Result<T, LoadPbmErr>;

#[derive(Debug)]
pub enum LoadPbmErr {
    HeaderError(String),
    MissingSizeError,
    InvalidSizeError(String),
}

impl PBM {
    pub fn from_ascii(string: &str) -> PbmResult<PBM> {
        let mut characters = string.split_whitespace();
        let flag = characters.next();
        match flag {
            Some("P1") => {}
            Some(bad) => {
                return Err(LoadPbmErr::HeaderError(
                    "PBM file did not contain starting header of P1 contains:".to_owned() + bad,
                ));
            }
            None => {
                return Err(LoadPbmErr::HeaderError(
                    "PBM file did not contain starting header.".to_owned(),
                ));
            }
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
                return Err(LoadPbmErr::InvalidSizeError(
                    "Failed to convert size line into u16".to_owned() + &error_string,
                ));
            }
        };

        let cells = characters
            .filter_map(|c| match c {
                "0" => Some(false),
                "1" => Some(true),
                _ => None,
            })
            .collect();

        Ok(PBM {
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

        if let Ok(pbm) = PBM::from_ascii(&data) {
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
            panic!("Failed to load PBM file, got {:?}", PBM::from_ascii(&data));
        }
    }
}
