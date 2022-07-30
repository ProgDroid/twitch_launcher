#[allow(clippy::integer_arithmetic)]
pub const fn index_add(current_value: usize, size: usize) -> usize {
    (current_value + 1) % size
}

#[allow(clippy::integer_arithmetic)]
pub const fn index_subtract(current_value: usize, size: usize) -> usize {
    (current_value + size - 1) % size
}
