//! Language-specific comment pattern detection.
//!
//! This module defines comment patterns for various programming languages
//! to enable polyglot TODO/FIXME scanning.

use std::path::Path;

/// Language configuration for comment detection
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// Language name
    pub name: &'static str,

    /// File extensions
    pub extensions: &'static [&'static str],

    /// Line comment patterns
    pub line_comment: &'static [&'static str],

    /// Block comment start pattern (if supported)
    pub block_comment_start: Option<&'static str>,

    /// Block comment end pattern (if supported)
    pub block_comment_end: Option<&'static str>,
}

impl LanguageConfig {
    /// Detect language from file extension
    pub fn from_path(path: &Path) -> Option<&'static LanguageConfig> {
        let extension = path.extension()?.to_str()?;
        let ext_with_dot = format!(".{extension}");

        LANGUAGES
            .iter()
            .find(|lang| lang.extensions.contains(&ext_with_dot.as_str()))
    }

    /// Check if a line could be a comment in this language
    pub fn could_be_comment(&self, line: &str) -> bool {
        let trimmed = line.trim_start();

        // Check line comments
        for pattern in self.line_comment {
            if trimmed.starts_with(pattern) {
                return true;
            }
        }

        // Check block comments
        if let Some(start) = self.block_comment_start {
            if trimmed.starts_with(start) {
                return true;
            }
        }

        false
    }
}

/// Rust language configuration
pub const RUST: LanguageConfig = LanguageConfig {
    name: "rust",
    extensions: &[".rs"],
    line_comment: &["//", "///", "//!"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

/// TypeScript/JavaScript language configuration
pub const TYPESCRIPT: LanguageConfig = LanguageConfig {
    name: "typescript",
    extensions: &[".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"],
    line_comment: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

/// Python language configuration
pub const PYTHON: LanguageConfig = LanguageConfig {
    name: "python",
    extensions: &[".py", ".pyi"],
    line_comment: &["#"],
    block_comment_start: None,
    block_comment_end: None,
};

/// Go language configuration
pub const GO: LanguageConfig = LanguageConfig {
    name: "go",
    extensions: &[".go"],
    line_comment: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

/// C/C++ language configuration
pub const C_CPP: LanguageConfig = LanguageConfig {
    name: "c_cpp",
    extensions: &[".c", ".cpp", ".cc", ".cxx", ".h", ".hpp"],
    line_comment: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

/// Java language configuration
pub const JAVA: LanguageConfig = LanguageConfig {
    name: "java",
    extensions: &[".java"],
    line_comment: &["//"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

/// Ruby language configuration
pub const RUBY: LanguageConfig = LanguageConfig {
    name: "ruby",
    extensions: &[".rb"],
    line_comment: &["#"],
    block_comment_start: None,
    block_comment_end: None,
};

/// Shell script language configuration
pub const SHELL: LanguageConfig = LanguageConfig {
    name: "shell",
    extensions: &[".sh", ".bash", ".zsh"],
    line_comment: &["#"],
    block_comment_start: None,
    block_comment_end: None,
};

/// Generic fallback configuration
pub const GENERIC: LanguageConfig = LanguageConfig {
    name: "generic",
    extensions: &[],
    line_comment: &["//", "#", "--"],
    block_comment_start: Some("/*"),
    block_comment_end: Some("*/"),
};

/// All supported languages
pub const LANGUAGES: &[LanguageConfig] = &[RUST, TYPESCRIPT, PYTHON, GO, C_CPP, JAVA, RUBY, SHELL];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        let path = Path::new("src/main.rs");
        let lang = LanguageConfig::from_path(path).unwrap();
        assert_eq!(lang.name, "rust");
    }

    #[test]
    fn test_detect_typescript() {
        let path = Path::new("src/component.tsx");
        let lang = LanguageConfig::from_path(path).unwrap();
        assert_eq!(lang.name, "typescript");
    }

    #[test]
    fn test_detect_python() {
        let path = Path::new("script.py");
        let lang = LanguageConfig::from_path(path).unwrap();
        assert_eq!(lang.name, "python");
    }

    #[test]
    fn test_could_be_comment_rust() {
        assert!(RUST.could_be_comment("// TODO: test"));
        assert!(RUST.could_be_comment("  // FIXME: broken"));
        assert!(RUST.could_be_comment("/// Doc comment"));
        assert!(RUST.could_be_comment("/* Block comment"));
        assert!(!RUST.could_be_comment("let x = 5;"));
    }

    #[test]
    fn test_could_be_comment_python() {
        assert!(PYTHON.could_be_comment("# TODO: test"));
        assert!(PYTHON.could_be_comment("  # FIXME: broken"));
        assert!(!PYTHON.could_be_comment("def main():"));
    }

    #[test]
    fn test_could_be_comment_typescript() {
        assert!(TYPESCRIPT.could_be_comment("// TODO: test"));
        assert!(TYPESCRIPT.could_be_comment("/* FIXME: broken */"));
        assert!(!TYPESCRIPT.could_be_comment("const x = 5;"));
    }
}
