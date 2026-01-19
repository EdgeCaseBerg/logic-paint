use crate::netbpm::Pbm;

#[derive(Debug, PartialEq)]
pub struct Group {
    pub num_cells: usize,
    pub filled: bool,
}

#[derive(Debug)]
pub struct PlayState {
    player_moves: Pbm,
    column_groups: Vec<Vec<Group>>,
    row_groups: Vec<Vec<Group>>,
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

impl From<&Pbm> for PlayState {
    fn from(pbm: &Pbm) -> PlayState {
        let empty_pbm = Pbm {
            width: pbm.width,
            height: pbm.height,
            cells: pbm.cells.iter().map(|_| false).collect(),
        };

        PlayState {
            player_moves: empty_pbm,
            column_groups: groups(&pbm.cols()),
            row_groups: groups(&pbm.rows()),
        }
    }
}

impl PlayState {
    pub fn update_groups(&mut self, _truth: &Pbm) {
        todo!();
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
            state.player_moves.cells.iter().all(|b| !*b),
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
}
