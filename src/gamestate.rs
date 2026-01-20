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

    pub fn row_goal_pairs(&self) -> Vec<Vec<(PlayerSetState, GoalState)>> {
        let mut result = vec![];

        let pairs: Vec<(CellState, CellState)> = zip(
            self.cells.clone().into_iter(),
            self.goal_state.clone().into_iter(),
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

    pub fn update_groups(&mut self) {
        // TODO: should we validate the cells against goal_state first?
        //       or just leave that as a different thing that's always done before this is called?
        self.row_groups = groups_from_goal_pairs(&self.row_goal_pairs());
        self.column_groups = groups_from_goal_pairs(&self.column_goal_pairs());
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

// TODO Should we just use a From?
// https://youtu.be/gkIpRTq1S6A
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

        let state: PlayState = (&pbm).into();
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

        let mut state: PlayState = (&pbm).into();

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
}
