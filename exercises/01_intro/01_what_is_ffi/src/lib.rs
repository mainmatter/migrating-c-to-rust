fn add(a: i32, b: i32) -> i32 {
    // write an `extern "C" {}` block declaring the `bm_add` function we want to bind to
    // then call this function here to implement the add!

    todo!()
}

#[cfg(test)]
mod tests {
    use super::add;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(0, 0), 0);
    }
}
