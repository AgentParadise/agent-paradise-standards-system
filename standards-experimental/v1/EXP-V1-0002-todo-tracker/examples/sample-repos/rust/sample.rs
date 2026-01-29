// Sample Rust file with TODO/FIXME comments for testing

pub mod auth {
    // TODO(#123): Add integration tests with real repository
    pub fn validate_token(token: &str) -> Result<(), Error> {
        // FIXME(#456): This breaks with empty input
        let decoded = decode_jwt(token)?;
        Ok(())
    }
    
    /// TODO(#789): Add rate limiting support
    /// This is a multi-line TODO that should only
    /// match on the first line
    pub fn process_request() {
        // TODO: Add error handling (missing issue reference)
    }
}

/* FIXME(#101): Memory leak in cache implementation
   This needs immediate attention */
fn cache_data() {}

#[cfg(test)]
mod tests {
    // TODO(#202): Write more comprehensive tests
    #[test]
    fn test_basic() {}
}
