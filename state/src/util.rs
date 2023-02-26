pub const fn index_add(current_value: usize, size: usize) -> usize {
    match size {
        0 => current_value,
        _ => (current_value + 1) % size,
    }
}

pub const fn index_subtract(current_value: usize, size: usize) -> usize {
    match size {
        0 => current_value,
        _ => (current_value + size - 1) % size,
    }
}
