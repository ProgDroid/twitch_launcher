mod app_state;
pub mod cache;
mod event;
mod input_mappings;
pub mod state;
pub mod state_machine;
pub mod transition;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
