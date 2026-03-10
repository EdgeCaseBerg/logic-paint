use crate::editor::editor_grids::EditorGrids;
use crate::editor::editor_settings::LevelSettings;
use crate::editor::solver::SolvedState;
use crate::editor::solver::TheMultiVerseOfLines;
use crate::netpbm::Pbm;
use crate::playstate::PlayState;
use egor::app::egui::Ui;

pub struct SolverDisplay {
    pub iterations: usize,
    pub state: SolvedState,
}

impl Default for SolverDisplay {
    fn default() -> Self {
        Self {
            iterations: 0,
            state: SolvedState::UniqueSolution,
        }
    }
}

impl SolverDisplay {
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.heading("Level Correctness");
        let state = match self.state {
            SolvedState::UniqueSolution => "Good: No guesses needed!",
            SolvedState::Unsolvable => "Bad: Unsolvable",
            SolvedState::MultipleSolutions => "Bad: Guessing or multiple solutions found.",
        };
        ui.label(state);
        ui.label(format!("Solved in {}", self.iterations));
        ui.label("If multiple solutions are possible, or if there is no solution,");
        ui.label("corresponding rows and columns will be highlighted.");
        ui.label("The number shown is the number of potential solutions.");
        ui.label("If your puzzle is good, the highlight will be green.");
        ui.separator();
    }

    pub fn recompute(
        &mut self,
        level_settings: &LevelSettings,
        editor_grids: &EditorGrids,
    ) -> TheMultiVerseOfLines {
        let pbm: Pbm = (level_settings, editor_grids).into();
        let ps: PlayState = (&pbm).into();
        let mut possibilities = TheMultiVerseOfLines::new(&ps);
        self.iterations = possibilities.collapse();
        self.state = possibilities.state();
        possibilities
    }
}
