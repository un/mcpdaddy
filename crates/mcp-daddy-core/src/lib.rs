pub const APP_NAME: &str = "mcp-daddy";

pub fn build_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!build_version().is_empty());
    }
}
