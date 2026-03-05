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

pub fn generate_line_pattern(remaining_space: usize, groups: &[usize]) -> Vec<LinePattern> {
    // for the groups we we can generate the spot the first one should be at
    // up to the space required for the other groups, aka, if we have 2,1 then
    // x x _ _ _ _ _ [reserved] and then shift the xx along to the right. Then,
    // I suppose each of those acts as a base for where the reserved patterns
    // can do the same, so each time the space to wiggle and jiggle stays the
    // same.
    // Base case: we have no more groups to consider, run.
    if groups.len() == 0 || remaining_space == 0 {
        return vec![];
    }

    let Some((group, others)) = groups.split_at_checked(1) else {
        return vec![];
    };
    eprintln!("{:?} {:?}", group, others);

    let size_of_first_group = group[0];
    let other_patterns = generate_line_pattern(
        remaining_space
            .saturating_sub(size_of_first_group)
            .saturating_sub(1),
        others,
    );
    // other patterns will be smaller by remaining space, and so will need to be shifted to the correct place
    // to be combined with any patterns we construct from the current group.
    // so, lets make the bits for the current group!
    let one_bit_on_the_left = u32::MAX ^ (u32::MAX >> 1);
    let mut pattern = 0;
    for _ in 0..size_of_first_group {
        let shifted = pattern >> 1;
        pattern = one_bit_on_the_left | shifted;
    }
    eprintln!("{:032b}", pattern);

    // K, now I've got 111000 with a left aligned block. So, how much space needs to be reserved
    // on the right hand side that I shouldn't touch?
    let reserved = match others.len() {
        0 => 0,
        remaining_groups => others.iter().sum::<usize>() + remaining_groups - 1,
    };
    let mut patterns = Vec::with_capacity(remaining_space);
    // for each potential shift to the right...
    eprintln!("{} {}", remaining_space, reserved);
    for _ in 0..remaining_space.saturating_sub(reserved) {
        // combine it with each potential pattern of the other patterns
        for other_pattern in &other_patterns {
            let other_pattern = other_pattern >> reserved;
            // We need to shift the other pattern down
            patterns.push(pattern | other_pattern);
        }
        if other_patterns.is_empty() {
            patterns.push(pattern);
        }
        eprintln!("{:032b}", pattern);
        pattern = pattern >> 1;
    }
    eprintln!("patterns: {:?}", patterns);

    patterns
}

#[cfg(test)]
mod pbm_tests {
    use super::*;

    fn print_patterns(patterns: &[u32]) {
        for pattern in patterns {
            eprintln!("{:032b}", pattern);
        }
    }

    #[test]
    fn can_gen_a_10_and_01_type_pattern() {
        let mut patterns = generate_line_pattern(2, &[1]);
        patterns.sort();
        assert_eq!(patterns.len(), 2);
        print_patterns(&patterns);
        // 0100...
        assert_eq!((u32::MAX ^ (u32::MAX >> 1)) >> 1, patterns[0]);
        // 1000...
        assert_eq!(u32::MAX ^ (u32::MAX >> 1), patterns[1]);
    }

    #[test]
    fn can_gen_the_only_option_as_needed() {
        let mut patterns = generate_line_pattern(1, &[1]);
        patterns.sort();
        print_patterns(&patterns);
        assert_eq!(patterns.len(), 1);
        // 1000...
        assert_eq!(u32::MAX ^ (u32::MAX >> 1), patterns[0]);
    }
}
