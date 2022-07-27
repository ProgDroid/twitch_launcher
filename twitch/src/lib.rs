#![allow(clippy::pub_use)]

pub mod account;
pub mod channel;
pub use channel::status;
mod secret;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
