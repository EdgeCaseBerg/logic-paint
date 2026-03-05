use crate::editor::editor_grids::EditorGrids;
use crate::editor::editor_settings::LevelSettings;
use crate::netbpm::Pbm;
use crate::playstate::PlayState;

pub enum SolvedState {
    Unsolvable,
    UniqueSolution,
    MultipleSolutions,
}

pub fn editor_to_initial_state(level_settings: &LevelSettings, grids: &EditorGrids) -> PlayState {
    let pbm: Pbm = (level_settings, grids).into();
    (&pbm).into()
}

type LinePattern = u32;

// is_empty p == 0
// is_full? p == u32::MAX

pub fn generate_line_pattern(size: usize, groups: &[usize]) -> Vec<LinePattern> {
    // TODO: Shift the bits around wahoo
    vec![2147483648]
}

#[cfg(test)]
mod pbm_tests {
    use super::*;

    #[test]
    fn name() {
        let patterns = generate_line_pattern(1, &[1]);
        assert_eq!(patterns.len(), 1);
        //2147483648 aka 10000000000000000000000000000000
        assert_eq!(u32::MAX ^ (u32::MAX >> 1), patterns[0]);
    }
}
