use crate::editor::editor_grids::EditorGrids;
use crate::editor::editor_settings::LevelSettings;
use crate::netbpm::Pbm;
use crate::playstate::PlayState;

#[derive(PartialEq, Debug)]
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
const MAX_BITS: usize = 32; // this should match to LinePattern. Always!

pub fn bitblock_of(size: usize, at: usize) -> LinePattern {
    assert!(at < MAX_BITS);
    let one_bit_on_the_left = u32::MAX ^ (u32::MAX >> 1);
    let mut base_pattern = 0;
    for _ in 0..size {
        base_pattern = (base_pattern >> 1) | one_bit_on_the_left;
    }
    assert!(size + at - 1 < MAX_BITS);
    base_pattern = base_pattern >> at;
    base_pattern
}

// Remaining space is how much inclaimed space lays before our army of bits eagerly ready
// to conquer and claim the electric homes of their permutious multitude.
pub fn generate_line_pattern(remaining_space: usize, groups: &[usize]) -> Vec<LinePattern> {
    // If no groups remain, the line is empty so it can be filled as desired by the caller
    if groups.is_empty() {
        return vec![0];
    }

    // If there is no space, no places exist for the bits to make their home.
    // Solemnly cry out to the void and beg for a place to lay thy head only
    // to be cast aside with an empty space offered in return.
    if remaining_space == 0 {
        return vec![];
    }

    // If we can't split, then there isn't anything TO consider. Begone.
    // (Note this is actually redundant to our first case but I dont want the None so...)
    let Some((group, others)) = groups.split_first() else {
        assert!(false, "This code will never be reached");
        return vec![0];
    };

    let size_of_first_group = *group;
    if size_of_first_group == remaining_space {
        return vec![bitblock_of(size_of_first_group, 0)];
    }

    // Minimum space required for remaining groups, when this is empty
    // then we can let the bits run rampant to the far side and plant their
    // flag upon those fertile hills.
    let reserved = if others.is_empty() {
        0
    } else {
        others.iter().sum::<usize>() + (others.len() - 1)
    };

    // Construct left-aligned block for the first group
    let base_pattern = bitblock_of(size_of_first_group, 0);
    let mut patterns = Vec::new();

    // How far we can shift the first group
    let max_shift = remaining_space
        .saturating_sub(size_of_first_group)
        .saturating_sub(reserved);

    for inset in 0..=max_shift {
        // If there is no space at all, then skip.
        if max_shift == 0 {
            continue;
        }
        let pattern = base_pattern >> inset;

        // Remaining space AFTER placing first group + separator
        let remaining_after_group = remaining_space
            .saturating_sub(inset)
            .saturating_sub(size_of_first_group)
            .saturating_sub(if others.is_empty() { 0 } else { 1 });

        let other_patterns = generate_line_pattern(remaining_after_group, others);
        let shift = inset + size_of_first_group + 1;
        for other in other_patterns {
            patterns.push(pattern | (other >> shift));
        }
    }

    patterns
}

#[derive(Debug)]
pub struct TheMultiVerseOfLines {
    pub rows: Vec<Vec<LinePattern>>,
    pub columns: Vec<Vec<LinePattern>>,
}

impl TheMultiVerseOfLines {
    pub fn new(play_state: &PlayState) -> Self {
        let mut multiverse = Self {
            rows: Vec::new(),
            columns: Vec::new(),
        };
        for r in 0..play_state.num_rows {
            let row_groups: Vec<usize> = play_state.row_groups[r]
                .iter()
                .map(|g| g.num_cells)
                .collect();
            let row_patterns = generate_line_pattern(play_state.num_columns, &row_groups);
            multiverse.rows.push(row_patterns);
        }
        for c in 0..play_state.num_columns {
            let column_groups: Vec<usize> = play_state.column_groups[c]
                .iter()
                .map(|g| g.num_cells)
                .collect();
            let column_patterns = generate_line_pattern(play_state.num_rows, &column_groups);
            multiverse.columns.push(column_patterns);
        }
        multiverse
    }

    // known_filled = 1's where 1s are in row.
    // known_empty = 1's where 0s are in row
    pub fn get_assured_row_cells(&self, row_idx: usize) -> (LinePattern, LinePattern) {
        Self::assured_cells(&self.rows[row_idx])
    }

    pub fn get_assured_column_cells(&self, column_idx: usize) -> (LinePattern, LinePattern) {
        Self::assured_cells(&self.columns[column_idx])
    }

    fn assured_cells(patterns: &[LinePattern]) -> (LinePattern, LinePattern) {
        let mut all_filled_together = LinePattern::MAX;
        let mut all_empty_together = 0;

        for &pattern in patterns {
            all_filled_together &= pattern;
            all_empty_together |= pattern;
        }

        let must_be_filled = all_filled_together;
        let must_be_empty = !all_empty_together;
        (must_be_filled, must_be_empty)
    }

    pub fn collapse(&mut self) {
        loop {
            let mut changed = false;
            changed = changed || self.collapse_rows();
            changed = changed || self.collapse_columns();
            if !changed {
                break;
            }
        }
    }

    fn collapse_rows(&mut self) -> bool {
        Self::collapse_lines(&self.rows, &mut self.columns)
    }

    fn collapse_columns(&mut self) -> bool {
        Self::collapse_lines(&self.columns, &mut self.rows)
    }

    fn collapse_lines(lines: &[Vec<LinePattern>], patterns: &mut [Vec<LinePattern>]) -> bool {
        let mut changed = false;
        for l in 0..lines.len() {
            let (must_be_filled, must_be_empty) = Self::assured_cells(&lines[l]);
            for p in 0..patterns.len() {
                let valid_patterns = patterns[p].len();
                // This is no longer row and bit, but just source and target sorta, or line and pattern idx
                let line_idx = MAX_BITS - 1 - l;
                let pattern_idx = MAX_BITS - 1 - p;
                patterns[p].retain(|&pattern| {
                    if (must_be_filled >> pattern_idx) & 1 == 1 {
                        let fill_agree =
                            bit_agreed_at(must_be_filled, pattern, pattern_idx, line_idx);
                        if !fill_agree {
                            return false;
                        }
                    }
                    if (must_be_empty >> pattern_idx) & 1 == 1 {
                        let empty_agree =
                            bit_agreed_at(must_be_empty, !pattern, pattern_idx, line_idx);
                        if !empty_agree {
                            return false;
                        }
                    }
                    true
                });
                changed = changed || valid_patterns != patterns[p].len();
            }
        }
        changed
    }

    pub fn state(&self) -> SolvedState {
        // If we collapsed and found no options, there is a contradiction
        // or guesswork required, so fail it because we only want line solveable
        // puzzles!
        if self.rows.iter().any(|p| p.is_empty()) {
            return SolvedState::Unsolvable;
        }
        if self.columns.iter().any(|p| p.is_empty()) {
            return SolvedState::Unsolvable;
        }

        // But if each has a unique pattern left, then it is solveable with only 1 way!
        for pattern in &self.rows {
            if pattern.len() > 1 {
                return SolvedState::MultipleSolutions;
            }
        }
        for pattern in &self.columns {
            if pattern.len() > 1 {
                return SolvedState::MultipleSolutions;
            }
        }
        SolvedState::UniqueSolution
    }
}

pub fn bit_agreed_at(line1: LinePattern, line2: LinePattern, idx_1: usize, idx_2: usize) -> bool {
    let bit1 = (line1 >> idx_1) & 1;
    let bit2 = (line2 >> idx_2) & 1;
    bit1 == bit2
}

impl std::fmt::Display for TheMultiVerseOfLines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Rows:\n")?;
        for patterns in &self.rows {
            write!(f, "Row:\n")?;
            for line in patterns {
                write!(f, "{:032b}\n", line)?;
            }
        }
        write!(f, "Columns:\n")?;
        for patterns in &self.columns {
            write!(f, "Column:\n")?;
            for line in patterns {
                write!(f, "{:032b}\n", line)?;
            }
        }
        Ok(())
    }
}

/* The tests are beneath here and I like having something to cleanly
   separate the code and the tests for easy scanning.
=====================================================================
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣤⣾⡿⠿⢿⣦⡀⠀⠀⠀⠀⠀⠀
⠀⠀⢀⣶⣿⣶⣶⣶⣦⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⣿⠟⠁⣀⣤⡄ ⢹⣷⡀⠀⠀⠀⠀⠀
⠀⠀⢸⣿⡧⠤⠤⣌⣉⣩⣿⡿⠶⠶⠒⠛⠛⠻⠿⠶⣾⣿⣣⠔⠉⠀⠀⠙⡆ ⢻⣷⠀⠀⠀⠀⠀
⠀⠀⢸⣿⠀⠀⢠⣾⠟⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⡃⠀⠀⠀⠀⠀⢻ ⠘⣿⡀⠀⠀⠀⠀
⠀⠀⠘⣿⡀⣴⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠻⢶⣤⣀⠀⢘ ⠀⣿⡇⠀⠀⠀⠀
⠀⠀⠀⢿⣿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠛⢿ ⣴⣿⠀⠀⠀⠀⠀
⠀⠀⠀⣸⡟⠀⠀⠀⣴⡆⠀⠀⠀⠀⠀⠀⠀⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⣷⡀⠀⠀⠀⠀    _______________
⠀⠀⢰⣿⠁⠀⠀⣰⠿⣇⠀⠀⠀⠀⠀⠀⠀⢻⣷⡀⠀⢠⡄⠀⠀⠀⠀⠀⡀⠀⠹⣷⠀⠀⠀⠀   |
⠀⠀⣾⡏⠀⢀⣴⣿⣤⢿⡄⠀⠀⠀⠀⠀⠀⠸⣿⣷⡀⠘⣧⠀⠀⠀⠀⠀⣷⣄⠀⢻⣇⠀⠀⠀  /  Here be tests |
⠀⠀⢻⣇⠀⢸⡇⠀⠀⠀⢻⣄⠀⠀⠀⠀⠀⣤⡯⠈⢻⣄⢻⡄⠀⠀⠀⠀⣿⡿⣷⡌⣿⡄⠀⠀  \
⠀⢀⣸⣿⠀⢸⡇⣶⣶⡄⠀⠙⠛⠛⠛⠛⠛⠃⣠⣶⣄⠙⠿⣧⠀⠀⠀⢠⣿⢹⣻⡇⠸⣿⡄⠀   |_______________|
⢰⣿⢟⣿⡴⠞⠀⠘⢿⡿⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣿⡇⠀⣿⡀⢀⣴⠿⣿⣦⣿⠃⠀⢹⣷⠀
⠀⢿⣿⠁⠀⠀⠀⠀⠀⠀⠀⢠⣀⣀⡀⠀⡀⠀⠀⠀⠀⠀⠀⣿⠛⠛⠁⠀⣿⡟⠁⠀⠀⢀⣿
⠀ ⠛⢷⣤⣀⠀⠀⠀⠀⠀⠀⠉⠉⠉⠛⠉⠀⠀⠀⠀⠀⢠⡿⢰⡟⠻⠞⠛⣧⣠⣦⣀⣾⠏⠀
⠀  ⠀⠈⠛⠛⠶⢶⣤⣤⣤⣤⣤⣤⣤⣤⣶⠶⠶⠛⠛⠛⠷⢾⣧⣠⡿⢿⡟⠋⠛⠋⠁⠀⠀

=====================================================================
*/

#[cfg(test)]
mod solver_tests {
    use super::*;

    fn print_patterns(patterns: &[u32]) {
        eprintln!("Debug pattern list:");
        for pattern in patterns {
            eprintln!("{:032b}", pattern);
        }
    }

    #[test]
    fn bitblock_works_as_expected() {
        let block = bitblock_of(1, 31);
        assert_eq!(0b00000000000000000000000000000001, block);
        let block = bitblock_of(2, 30);
        assert_eq!(0b00000000000000000000000000000011, block);
        let block = bitblock_of(2, 2);
        assert_eq!(0b00110000000000000000000000000000, block);
        let block = bitblock_of(32, 0);
        assert_eq!(0b11111111111111111111111111111111, block);
    }

    #[should_panic]
    #[test]
    fn bitblock_panics_as_expected() {
        bitblock_of(33, 0);
    }

    #[test]
    fn empty_line_handled_correctly() {
        let empty: Vec<usize> = vec![];
        let patterns = generate_line_pattern(1, &empty);
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 1);
        assert_eq!(0, patterns[0]);
    }

    #[test]
    fn can_gen_the_only_option_as_needed() {
        let mut patterns = generate_line_pattern(1, &[1]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 1);
        // 1000...
        assert_eq!(bitblock_of(1, 0), patterns[0]);
    }

    #[test]
    fn can_gen_a_10_and_01_type_pattern() {
        let mut patterns = generate_line_pattern(2, &[1]);
        patterns.sort();
        assert_eq!(patterns.len(), 2);
        print_patterns(&patterns);
        // 0100...
        assert_eq!(bitblock_of(1, 1), patterns[0]);
        // 1000...
        assert_eq!(bitblock_of(1, 0), patterns[1]);
    }

    #[test]
    fn can_gen_a_1_across_3_cells_pattern() {
        let mut patterns = generate_line_pattern(3, &[1]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 3);
        // 001...
        assert_eq!(bitblock_of(1, 2), patterns[0]);
        // 010...
        assert_eq!(bitblock_of(1, 1), patterns[1]);
        // 100...
        assert_eq!(bitblock_of(1, 0), patterns[2]);
    }

    #[test]
    fn can_gen_two_1s_in_a_3_pattern() {
        let mut patterns = generate_line_pattern(3, &[1, 1]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 1);
        // 101
        let one_in_3rd_place = bitblock_of(1, 2);
        let one_in_1st_place = bitblock_of(1, 0);
        assert_eq!(one_in_1st_place | one_in_3rd_place, patterns[0]);
    }

    #[test]
    fn can_gen_two_groups_in_five_example() {
        let mut patterns = generate_line_pattern(5, &[2, 1]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 3);
        let one_in_1st_place = bitblock_of(1, 0);
        let one_in_2nd_place = bitblock_of(1, 1);
        let one_in_3rd_place = bitblock_of(1, 2);
        let one_in_4th_place = bitblock_of(1, 3);
        let one_in_5th_place = bitblock_of(1, 4);
        /*  Only valid patterns are:
            11010
            11001
            01101
        */
        let one_one_zero_one_zero = one_in_1st_place | one_in_2nd_place | one_in_4th_place;
        let one_one_zero_zero_one = one_in_1st_place | one_in_2nd_place | one_in_5th_place;
        let zero_one_one_zero_one = one_in_2nd_place | one_in_3rd_place | one_in_5th_place;
        assert_eq!(zero_one_one_zero_one, patterns[0]);
        assert_eq!(one_one_zero_zero_one, patterns[1]);
        assert_eq!(one_one_zero_one_zero, patterns[2]);
    }

    #[test]
    fn can_gen_three_1s_in_a_5_pattern() {
        let mut patterns = generate_line_pattern(5, &[1, 1, 1]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 1);
        // 101
        let one_in_5th_place = bitblock_of(1, 4);
        let one_in_3rd_place = bitblock_of(1, 2);
        let one_in_1st_place = bitblock_of(1, 0);
        assert_eq!(
            one_in_1st_place | one_in_3rd_place | one_in_5th_place,
            patterns[0]
        );
    }

    #[test]
    fn can_gen_two_2s_in_a_5_pattern() {
        let mut patterns = generate_line_pattern(5, &[2, 2]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 1);
        // 101
        let one_in_1st_place = bitblock_of(1, 0);
        let one_in_2nd_place = bitblock_of(1, 1);
        let one_in_4th_place = bitblock_of(1, 3);
        let one_in_5th_place = bitblock_of(1, 4);
        assert_eq!(
            one_in_1st_place | one_in_2nd_place | one_in_4th_place | one_in_5th_place,
            patterns[0]
        );
    }

    #[test]
    fn can_gen_two_2s_in_a_6_pattern() {
        let mut patterns = generate_line_pattern(6, &[2, 2]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 3);
        // 101
        let first = bitblock_of(2, 0);
        let second = bitblock_of(2, 3);
        assert_eq!(first >> 1 | second >> 1, patterns[0]);
        assert_eq!(first | second >> 1, patterns[1]);
        assert_eq!(first | second, patterns[2]);
    }

    #[test]
    fn confirm_number_of_compositions_aligns_to_formula_test() {
        //(25 - 2 + 1) choose 1 aka
        let patterns = generate_line_pattern(25, &[1]);
        assert_eq!(patterns.len(), 25);

        //(25 - 2 + 1) choose 2 aka
        let patterns = generate_line_pattern(25, &[1, 1]);
        assert_eq!(patterns.len(), 276);

        //(25 - 3 + 1) choose 3 aka
        let patterns = generate_line_pattern(25, &[1, 1, 1]);
        assert_eq!(patterns.len(), 1771);

        //(25 - 4 + 1) choose 3 aka
        let patterns = generate_line_pattern(25, &[1, 2, 1]);
        assert_eq!(patterns.len(), 1540);
    }

    #[test]
    fn bit_agreed_at_works_as_expected() {
        assert!(bit_agreed_at(1, 1, 0, 0));
        let three_on_left_offset_one = bitblock_of(3, 1);
        let three_on_left_offset_two = bitblock_of(3, 2);
        // zero
        assert!(bit_agreed_at(
            three_on_left_offset_one,
            three_on_left_offset_two,
            31,
            31
        ));
        // 1 and 0 disagree
        assert!(!bit_agreed_at(
            three_on_left_offset_one,
            three_on_left_offset_two,
            30,
            30
        ));
        // aligned again with overlap
        assert!(bit_agreed_at(
            three_on_left_offset_one,
            three_on_left_offset_two,
            29,
            29
        ));
    }

    #[rustfmt::skip]
    fn test_play_state() -> PlayState {
        let pbm = Pbm {
            width: 5,
            height: 5,
            cells: vec![
                true , false, false, false , false,
                true , true , false, false , false,
                true , true , true , false , false,
                true , true , true , true  , false,
                true , true , true , true  , true,
            ]
        };
        (&pbm).into()
    }

    #[test] // dr strange called.
    fn generate_correct_multiverse() {
        let tps = test_play_state();
        let multiverse = TheMultiVerseOfLines::new(&tps);
        eprintln!("{multiverse}");
        // the possibilites for the cells decrease as we go down
        assert_eq!(5, multiverse.rows[0].len());
        assert_eq!(4, multiverse.rows[1].len());
        assert_eq!(3, multiverse.rows[2].len());
        assert_eq!(2, multiverse.rows[3].len());
        assert_eq!(1, multiverse.rows[4].len());

        // the possibilites for the columns increase as we go to the right
        assert_eq!(1, multiverse.columns[0].len());
        assert_eq!(2, multiverse.columns[1].len());
        assert_eq!(3, multiverse.columns[2].len());
        assert_eq!(4, multiverse.columns[3].len());
        assert_eq!(5, multiverse.columns[4].len());
    }

    #[test]
    fn get_assured_row_cells_3_in_5_pattern() {
        let tps = test_play_state();
        let multiverse = TheMultiVerseOfLines::new(&tps);
        let potential = multiverse.get_assured_row_cells(2);
        let must_fill = bitblock_of(1, 2);
        let empty_first_5 = LinePattern::MAX >> 5;
        // For debugging when fail:
        print_patterns(&[must_fill, potential.0, potential.1]);
        assert_eq!(must_fill, potential.0);
        assert_eq!(empty_first_5, potential.1);
    }

    #[test]
    fn get_assured_col_cells_3_in_5_pattern() {
        let tps = test_play_state();
        let multiverse = TheMultiVerseOfLines::new(&tps);
        let potential = multiverse.get_assured_column_cells(2);
        let must_fill = bitblock_of(1, 2);
        let empty_first_5 = LinePattern::MAX >> 5;
        // For debugging when fail:
        print_patterns(&[must_fill, potential.0, potential.1]);
        assert_eq!(must_fill, potential.0);
        assert_eq!(empty_first_5, potential.1);
    }

    #[test]
    fn get_no_assurances_about_anything_when_not_enough_constraints() {
        let tps = test_play_state();
        let multiverse = TheMultiVerseOfLines::new(&tps);
        let potential = multiverse.get_assured_row_cells(0);
        let must_fill = 0;
        let empty_first_5 = LinePattern::MAX >> 5;
        // For debugging when fail:
        print_patterns(&[must_fill, potential.0, potential.1]);
        assert_eq!(must_fill, potential.0);
        assert_eq!(empty_first_5, potential.1);
    }

    #[test]
    fn requires_guessing_multi_solution() {
        let tps: PlayState = {
            let pbm = Pbm {
                width: 2,
                height: 2,
                cells: vec![
                    true  , false,
                    false , true ,
                ]
            };
            (&pbm).into()
        };
        let mut multiverse = TheMultiVerseOfLines::new(&tps);
        multiverse.collapse();
        assert_eq!(SolvedState::MultipleSolutions, multiverse.state());
    }

    #[test]
    fn has_one_solution() {
        let tps = test_play_state();
        let mut multiverse = TheMultiVerseOfLines::new(&tps);
        multiverse.collapse();
        eprintln!("{multiverse}");
        assert_eq!(SolvedState::UniqueSolution, multiverse.state());
    }
}
