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
