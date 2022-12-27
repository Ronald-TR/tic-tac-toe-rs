pub mod tictactoe;

pub mod proto {
    tonic::include_proto!("tictactoe");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
