pub fn prev_power_of_two(x: usize) -> usize {
    1 << (8 * size_of::<usize>() - x.leading_zeros() as usize - 1)
}