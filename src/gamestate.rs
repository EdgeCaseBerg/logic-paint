use crate::netbpm::Pbm;
use std::iter::zip;

#[derive(Debug, PartialEq)]
pub struct Group {
    pub num_cells: usize,
    pub filled: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CellState {
    Empty,
    Filled,
    Incorrect,
    UserRuledOut,
    RuledOut,
}

impl CellState {
    pub fn to_goal(self, goal: CellState) -> CellState {
        match (self, goal) {
            (CellState::Empty, _) => CellState::RuledOut,
            (CellState::Filled, CellState::Filled) => CellState::Filled,
            (CellState::Filled, oops) => panic!(
                "despite filled groups, player set cell state did not match desired goal of {:?}",
                oops
            ),
            (CellState::Incorrect, _) => CellState::Incorrect,
            (CellState::UserRuledOut, _) => CellState::RuledOut,
            (CellState::RuledOut, _) => CellState::RuledOut,
        }
    }

    pub fn attempt_fill(&self, goal: CellState) -> CellState {
        if *self == goal {
            return goal;
        }
        use CellState::*;
        match (self, goal) {
            (Empty, Filled) => goal,
            (Empty, _) => Incorrect,
            (Filled, _) => Filled,
            (Incorrect, _) => Incorrect,
            (RuledOut, _) => RuledOut,
            (UserRuledOut, _) => UserRuledOut,
        }
    }

    // TODO: should this and attempt_file do &mut instead so we dont have to always assign a result?
    pub fn mark_cell(&self) -> CellState {
        use CellState::*;
        match *self {
            Empty => UserRuledOut,
            Filled => Filled,
            Incorrect => Incorrect,
            UserRuledOut => Empty,
            RuledOut => RuledOut,
        }
    }
}

type PlayerSetState = CellState;
type GoalState = CellState;

#[derive(Debug)]
pub struct PlayState {
    cells: Vec<CellState>,
    column_groups: Vec<Vec<Group>>,
    row_groups: Vec<Vec<Group>>,
    goal_state: Vec<CellState>,
    num_rows: usize,
    num_columns: usize,
}

// Simple, not full fields display, of PlayState for easy test debugging
impl std::fmt::Display for PlayState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        writeln!(f, "size: {} x {} ", self.num_rows, self.num_columns)?;
        writeln!(f, "cells:")?;
        for (r, row) in self.rows().into_iter().enumerate() {
            let group_numbers: String = self.row_groups[r]
                .iter()
                .map(|g| g.num_cells.to_string() + ",")
                .collect::<String>();
            write!(f, "{}", format!("{:>10} {:?}\n", group_numbers, row))?;
        }
        for r in 0..self.num_rows {
            let group_numbers: String = self.column_groups[r]
                .iter()
                .map(|g| g.num_cells.to_string() + ", ")
                .collect::<String>();
            write!(f, "{}", format!("{:>1}{:>10}", "|", group_numbers))?;
        }
        writeln!(f)?;

        writeln!(f, "goals:")?;
        for r in 0..self.num_rows {
            for c in 0..self.num_columns {
                write!(f, "{:>10?} ", self.goal_state[r * self.num_rows + c])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl From<&Pbm> for PlayState {
    fn from(pbm: &Pbm) -> PlayState {
        PlayState {
            cells: pbm.cells.iter().map(|_| CellState::Empty).collect(),
            goal_state: pbm
                .cells
                .iter()
                .map(|filled| match filled {
                    true => CellState::Filled,
                    false => CellState::Empty,
                })
                .collect(),
            column_groups: groups(&pbm.cols()),
            row_groups: groups(&pbm.rows()),
            num_rows: pbm.height,
            num_columns: pbm.width,
        }
    }
}

impl PlayState {
    pub fn rows(&self) -> Vec<Vec<CellState>> {
        let mut result = vec![];
        for chunk in self.cells.chunks(self.num_columns) {
            result.push(chunk.to_vec());
        }
        result
    }

    #[rustfmt::skip]
    pub fn row_goal_pairs(&self) -> Vec<Vec<(PlayerSetState, GoalState)>> {
        let mut result = vec![];

        let pairs: Vec<(CellState, CellState)> = zip(
            self.cells.clone(),
            self.goal_state.clone(),
        )
        .collect();
        for chunk in pairs.chunks(self.num_columns) {
            result.push(chunk.to_vec());
        }
        result
    }

    pub fn cols(&self) -> Vec<Vec<CellState>> {
        let mut cols = vec![vec![]; self.num_columns];
        for c in 0..self.num_columns {
            for row in self.rows() {
                cols[c].push(row[c]);
            }
        }
        cols
    }

    pub fn column_goal_pairs(&self) -> Vec<Vec<(PlayerSetState, GoalState)>> {
        let mut cols = vec![vec![]; self.num_columns];
        for c in 0..self.num_columns {
            for row in self.row_goal_pairs() {
                cols[c].push(row[c]);
            }
        }
        cols
    }

    pub fn is_complete(&self) -> bool {
        // assumes that groups have been computed at least once or else they'll all be empty
        if self.column_groups.is_empty() || self.row_groups.is_empty() {
            panic!("Called is_complete before groups were computed");
        }
        let all_columns_filled = self.column_groups.iter().flatten().all(|g| g.filled);
        let all_rows_filled = self.row_groups.iter().flatten().all(|g| g.filled);
        all_rows_filled && all_columns_filled
    }

    pub fn attempt_fill(&mut self, row: usize, column: usize) {
        if row >= self.num_rows || column >= self.num_columns {
            return;
        }
        let offset = row * self.num_rows + column;
        let goal = self.goal_state[offset];
        self.cells[offset] = self.cells[offset].attempt_fill(goal);
    }

    pub fn mark_cell(&mut self, row: usize, column: usize) {
        if row >= self.num_rows || column >= self.num_columns {
            return;
        }
        let offset = row * self.num_rows + column;
        self.cells[offset] = self.cells[offset].mark_cell();
    }

    pub fn number_incorrect(&self) -> usize {
        self.cells
            .iter()
            .filter(|&&cell| cell == CellState::Incorrect)
            .count()
    }

    pub fn update_groups(&mut self) {
        self.row_groups = groups_from_goal_pairs(&self.row_goal_pairs());
        self.column_groups = groups_from_goal_pairs(&self.column_goal_pairs());
        self.fill_in_completed_groups();
    }

    fn fill_in_completed_groups(&mut self) {
        self.fill_in_completed_row_groups();
        self.fill_incompleted_column_groups();
    }

    fn fill_in_completed_row_groups(&mut self) {
        let row_pairs = self.row_goal_pairs();

        for (row, groups) in self.row_groups.iter().enumerate() {
            let complete = groups.iter().all(|g| g.filled);
            if !complete {
                continue;
            }

            for (column, (state, goal)) in row_pairs[row].iter().enumerate() {
                let new_value = state.to_goal(*goal);
                self.cells[row * self.num_rows + column] = new_value;
            }
        }
    }

    fn fill_incompleted_column_groups(&mut self) {
        let column_pairs = self.column_goal_pairs();

        for (column, groups) in self.column_groups.iter().enumerate() {
            let complete = groups.iter().all(|g| g.filled);
            if !complete {
                continue;
            }

            for (row, (state, goal)) in column_pairs[column].iter().enumerate() {
                let new_value = state.to_goal(*goal);
                self.cells[row * self.num_rows + column] = new_value;
            }
        }
    }
}

fn groups(cells: &[Vec<bool>]) -> Vec<Vec<Group>> {
    cells
        .iter()
        .map(|row| {
            let groups: Vec<Group> = row
                .split(|boolean| !*boolean)
                .filter(|v| !v.is_empty())
                .map(|run| Group {
                    num_cells: run.len(),
                    filled: false,
                })
                .collect();

            if groups.is_empty() {
                vec![Group {
                    num_cells: 0,
                    filled: true,
                }]
            } else {
                groups
            }
        })
        .collect()
}

fn groups_from_goal_pairs(
    state_and_goal_pairs: &[Vec<(PlayerSetState, GoalState)>],
) -> Vec<Vec<Group>> {
    state_and_goal_pairs
        .iter()
        .map(|row| {
            if row.iter().all(|(_, goal)| *goal == CellState::Empty) {
                return vec![Group {
                    num_cells: 0,
                    filled: true,
                }];
            }

            let groups: Vec<Group> = row
                .split(|(_, goal)| *goal == CellState::Empty)
                .filter(|v| !v.is_empty())
                .map(|run| Group {
                    num_cells: run.len(),
                    filled: run.iter().all(|(state, _)| *state == CellState::Filled),
                })
                .collect();

            groups
        })
        .collect()
}

#[cfg(test)]
mod pbm_tests {
    use super::*;

    #[rustfmt::skip]
    fn test_play_state() -> PlayState {
        let pbm = Pbm {
            width: 5,
            height: 5,
            cells: vec![
                false, false, false, false, false,
                true , true , false, false ,true,
                true , true , true , true , true,
                true , false, true , false, true,
                true , false, false, true , true,
            ]
        };
        (&pbm).into()
    }

    fn empty_group() -> Vec<Group> {
        vec![Group {
            num_cells: 0,
            filled: true,
        }]
    }

    fn groups_of(counts: &[usize]) -> Vec<Group> {
        counts
            .into_iter()
            .map(|count| Group {
                num_cells: *count,
                filled: false,
            })
            .collect()
    }

    #[test]
    #[rustfmt::skip]
    fn constructs_row_groups_correctly() {
        let mut state = test_play_state();

        assert_eq!(
            true,
            state.cells.iter().all(|cell| *cell == CellState::Empty),
            "all cells start empty"
        );
        let expected_row_groups = vec![
            empty_group(),
            groups_of(&[2, 1]),
            groups_of(&[5]),
            groups_of(&[1, 1, 1]),
            groups_of(&[1, 2]),
        ];
        assert_eq!(expected_row_groups, state.row_groups);

        let expected_col_groups = vec![
            groups_of(&[4]),
            groups_of(&[2]),
            groups_of(&[2]),
            groups_of(&[1, 1]),
            groups_of(&[4]),
        ];
        assert_eq!(expected_col_groups, state.column_groups);
        assert_eq!(false, state.is_complete());
    }

    #[test]
    #[rustfmt::skip]
    fn validates_row_groups_correctly() {
        use CellState::*;

        let mut state = test_play_state();

        state.cells = vec![
            Empty , Empty, Empty, Empty, Empty ,
            Filled, Empty, Empty, Empty, Filled, // <-- we fill in 1/2 of group 1, and all of group 2
            Empty , Empty, Empty, Empty, Filled,
            Empty , Empty, Empty, Empty, Filled,
            Empty , Empty, Empty, Empty, Filled,
            //^--- not full column group   ^------ we fill in all of this column group
        ];

        eprintln!("BEFORE: {:?}", state);
        state.update_groups();
        eprintln!("AFTER: {:?}", state);

        assert_eq!(false, state.row_groups[1][0].filled);
        assert_eq!(true, state.row_groups[1][1].filled);

        assert_eq!(false, state.column_groups[0][0].filled);
        assert_eq!(true, state.column_groups[4][0].filled);
    }

    #[test]
    #[rustfmt::skip]
    fn fill_in_row_on_groups_complete() {
        use CellState::*;

        let mut state = test_play_state();

        state.cells = vec![
            Empty , Empty    , Empty       , Empty , Empty ,
            Filled, Filled   , Empty       , Empty , Filled,
            Empty , Empty    , Empty       , Empty , Empty ,
            Filled, Incorrect, Filled      , Empty , Filled,
            Filled, Empty    , UserRuledOut, Filled, Filled,
        ];

        state.update_groups();

        let expected = vec![
            RuledOut, RuledOut , RuledOut, RuledOut, RuledOut,
            Filled  , Filled   , RuledOut, RuledOut, Filled  ,
            Empty   , Empty    , Empty   , Empty   , Empty   ,
            Filled  , Incorrect, Filled  , RuledOut, Filled  ,
            Filled  , RuledOut , RuledOut, Filled  , Filled  ,
        ];

        assert_eq!(expected, state.cells);
    }

    #[test]
    #[rustfmt::skip]
    fn fill_in_cols_on_groups_complete() {
        use CellState::*;

        let mut state = test_play_state();

        state.cells = vec![
            Empty , Empty    , Empty       , Empty , Empty,
            Filled, Filled   , Empty       , Empty , Empty,
            Filled ,Filled   , Filled      , Empty , Empty,
            Filled, Incorrect, Filled      , Empty , Empty,
            Filled, Empty    , UserRuledOut, Empty , Empty,
        ];

        state.update_groups();

        let expected = vec![
            RuledOut, RuledOut  , RuledOut , RuledOut , RuledOut,
            Filled  , Filled    , RuledOut , Empty    , Empty,
            Filled  , Filled    , Filled   , Empty    , Empty,
            Filled  , Incorrect , Filled   , Empty    , Empty,
            Filled  , RuledOut  , RuledOut , Empty    , Empty,
        ];

        assert_eq!(expected, state.cells);
    }

    #[test]
    fn can_fill_in_cell_that_is_empty() {
        let mut state = test_play_state();
        eprintln!("BEFORE: {}", state);
        state.cells[0] = CellState::Empty;
        state.goal_state[0] = CellState::Filled;
        state.attempt_fill(0, 0);
        eprintln!("AFTER: {}", state);
        assert_eq!(CellState::Filled, state.cells[0]);
    }

    #[test]
    fn cannot_fill_in_cell_that_is_marked() {
        let mut state = test_play_state();
        eprintln!("BEFORE: {}", state);
        state.cells[0] = CellState::UserRuledOut;
        state.goal_state[0] = CellState::Filled;
        state.attempt_fill(0, 0);
        eprintln!("AFTER: {}", state);
        assert_eq!(CellState::UserRuledOut, state.cells[0]);
    }

    #[test]
    fn can_mark_a_cell_to_rule_it_out() {
        let mut state = test_play_state();
        eprintln!("BEFORE: {}", state);
        state.cells[0] = CellState::Empty;
        state.goal_state[0] = CellState::Filled;
        state.mark_cell(0, 0);
        eprintln!("AFTER: {}", state);
        assert_eq!(CellState::UserRuledOut, state.cells[0]);
    }

    #[test]
    fn can_unmark_a_cell_to_clear_back_to_empty() {
        let mut state = test_play_state();
        eprintln!("BEFORE: {}", state);
        state.cells[0] = CellState::Empty;
        state.goal_state[0] = CellState::Filled;
        state.mark_cell(0, 0);
        eprintln!("AFTER: {}", state);
        assert_eq!(CellState::UserRuledOut, state.cells[0]);
        state.mark_cell(0, 0);
        assert_eq!(CellState::Empty, state.cells[0]);
    }

    #[test]
    fn should_do_nothing_if_asked_to_fill_oob_cell() {
        let mut state = test_play_state();
        let evil_row = state.num_rows + 1;
        let evil_column = state.num_columns + 1;
        state.attempt_fill(evil_row, 0);
        state.attempt_fill(evil_row, evil_column);
        state.attempt_fill(0, evil_column);
    }

    #[test]
    fn should_do_nothing_if_asked_to_mark_oob_cell() {
        let mut state = test_play_state();
        let evil_row = state.num_rows + 1;
        let evil_column = state.num_columns + 1;
        state.mark_cell(evil_row, 0);
        state.mark_cell(evil_row, evil_column);
        state.mark_cell(0, evil_column);
    }

    #[test]
    fn number_incorrect_returns_number_of_incorrect_cells() {
        let mut state = test_play_state();
        let mut count = 0;
        for (i, g) in state.goal_state.iter().enumerate() {
            match g {
                CellState::Empty => {
                    count += 1;
                    state.cells[i] = CellState::Incorrect
                }
                _ => {}
            }
        }
        assert_eq!(count, state.number_incorrect());
    }
}
