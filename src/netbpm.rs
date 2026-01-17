use std::fs::read_to_string;

#[derive(Debug)]
pub struct PBM {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<bool>,
}

impl BPM {
	pub fn from_ascii(string: &str) -> BPM {
		BPM {
			width: 0,
			height: 0,
			cells: Vec::new()
		}
	}
}


#[cfg(test)]
mod bpm_tests {
	use super::*;
	#[test]
	fn can_load_sample_ascii() {
		let data = read_to_string("assets/P1.pbm").expect("Could not load asset file for test (P1.pbm)");
		let parsed = BPM::from_ascii(&data);
		assert_eq!(parsed.width, 5); 
		assert_eq!(parsed.height, 5); 
		assert_eq!(parsed.cells, vec![
			false, true , true, true , false,
			false, true , true, true , false,
			false, false, true, false, false,
			false, true , true, true , false,
			false, true , true, true , false
		]);
	}
}