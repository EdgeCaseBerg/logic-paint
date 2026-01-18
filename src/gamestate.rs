use crate::netbpm::Pbm;

pub struct Group {
    pub num_cells: usize,
    pub filled: bool,
}

impl Default for Group {
    fn default() -> Group {
        Group {
            num_cells: 0,
            filled: true,
        }
    }
}

pub struct PlayState {
    player_moves: Pbm,
    column_groups: Vec<Group>,
    row_groups: Vec<Group>,
}

impl From<&Pbm> for PlayState {
    fn from(pbm: &Pbm) -> PlayState {
        let mut empty_pbm = Pbm {
            width: pbm.width,
            height: pbm.height,
            cells: pbm.cells.iter().map(|_| false).collect(),
        };

        // todo count and group
        let column_groups = vec![];
        let row_groups = vec![];

        PlayState {
            player_moves: empty_pbm,
            column_groups,
            row_groups,
        }
    }
}

impl PlayState {
    pub fn update_groups(&mut self, truth: &Pbm) {
        todo!();
    }

    pub fn is_complete(&self) -> bool {
        /// assumes that groups have been computed at least once or else they'll all be empty
        if self.column_groups.is_empty() || self.row_groups.is_empty() {
            panic!("Called is_complete before groups were computed");
        }
        self.column_groups.iter().all(|g| g.filled) && self.row_groups.iter().all(|g| g.filled)
    }
}
