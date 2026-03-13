//! Shared complexity calculation logic for all languages.
//!
//! This module provides unified implementations of:
//! - Cyclomatic Complexity (McCabe)
//! - Cognitive Complexity (SonarSource)
//! - Halstead Metrics
//!
//! All calculations use the grammar's configuration to determine which
//! AST node types contribute to complexity.

use std::collections::{HashMap, HashSet};

use tree_sitter::{Node, Tree};

use crate::{FunctionMetrics, HalsteadMetrics};

use super::grammars::Grammar;

// ============================================================================
// Complexity Calculator
// ============================================================================

/// Calculator for complexity metrics using tree-sitter ASTs.
///
/// The calculator uses the grammar's complexity rules to determine which
/// node types contribute to cyclomatic and cognitive complexity.
pub struct ComplexityCalculator<'g> {
    #[allow(dead_code)]
    grammar: &'g dyn Grammar,
    decision_nodes: HashSet<&'static str>,
    nesting_nodes: HashSet<&'static str>,
    ignored_nodes: HashSet<&'static str>,
}

impl<'g> ComplexityCalculator<'g> {
    /// Create a new calculator for the given grammar.
    pub fn new(grammar: &'g dyn Grammar) -> Self {
        Self {
            grammar,
            decision_nodes: grammar.decision_nodes().iter().copied().collect(),
            nesting_nodes: grammar.nesting_nodes().iter().copied().collect(),
            ignored_nodes: grammar.ignored_nodes().iter().copied().collect(),
        }
    }

    /// Compute all metrics for a function within the given line range.
    pub fn compute_metrics(
        &self,
        tree: &Tree,
        source: &[u8],
        start_line: u32,
        end_line: u32,
    ) -> FunctionMetrics {
        let root = tree.root_node();

        // Find the function node within the line range
        let function_node = self.find_node_in_range(root, start_line, end_line);

        let (cyclomatic, cognitive) = if let Some(node) = function_node {
            (
                self.compute_cyclomatic(node),
                self.compute_cognitive(node, 0),
            )
        } else {
            // Fallback: analyze entire range
            (
                self.compute_cyclomatic_range(root, start_line, end_line),
                self.compute_cognitive_range(root, start_line, end_line, 0),
            )
        };

        let halstead = self.compute_halstead(root, source, start_line, end_line);
        let (logical_lines, total_lines, comment_lines) =
            self.count_lines(source, start_line, end_line);

        FunctionMetrics {
            cyclomatic_complexity: cyclomatic,
            cognitive_complexity: cognitive,
            halstead,
            logical_lines,
            total_lines,
            comment_lines,
        }
    }

    /// Find a node that spans the given line range.
    fn find_node_in_range<'a>(
        &self,
        node: Node<'a>,
        start_line: u32,
        end_line: u32,
    ) -> Option<Node<'a>> {
        // Convert 0-indexed tree-sitter positions to 1-indexed line numbers
        let node_start = node.start_position().row as u32 + 1;
        let node_end = node.end_position().row as u32 + 1;

        // Check if this node approximately matches the range
        if node_start <= start_line && node_end >= end_line {
            // Check children for a tighter match
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(found) = self.find_node_in_range(child, start_line, end_line) {
                    return Some(found);
                }
            }
            // If this is a function-like node, return it
            if self.is_function_node(node.kind()) {
                return Some(node);
            }
        }

        None
    }

    /// Check if a node type represents a function.
    fn is_function_node(&self, kind: &str) -> bool {
        // Common function node types across languages
        matches!(
            kind,
            "function_item"
                | "function_definition"
                | "function_declaration"
                | "method_definition"
                | "arrow_function"
                | "lambda"
                | "closure_expression"
        )
    }

    // ========================================================================
    // Cyclomatic Complexity
    // ========================================================================

    /// Compute cyclomatic complexity for a node and its descendants.
    ///
    /// CC = 1 + number of decision points
    pub fn compute_cyclomatic(&self, node: Node) -> u32 {
        let mut cc = 1; // Base complexity

        self.visit_for_cyclomatic(node, &mut cc);

        cc
    }

    /// Compute cyclomatic complexity within a line range.
    fn compute_cyclomatic_range(&self, node: Node, start_line: u32, end_line: u32) -> u32 {
        let mut cc = 1;

        self.visit_for_cyclomatic_range(node, start_line, end_line, &mut cc);

        cc
    }

    fn visit_for_cyclomatic(&self, node: Node, cc: &mut u32) {
        let kind = node.kind();

        // Skip ignored nodes
        if self.ignored_nodes.contains(kind) {
            return;
        }

        // Check if this is a decision node
        if self.decision_nodes.contains(kind) {
            // Handle binary expressions specially (only count && and ||)
            if kind == "binary_expression" || kind == "boolean_operator" {
                if self.is_logical_operator(node) {
                    *cc += 1;
                }
            } else {
                *cc += 1;
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cyclomatic(child, cc);
        }
    }

    fn visit_for_cyclomatic_range(&self, node: Node, start_line: u32, end_line: u32, cc: &mut u32) {
        // Convert 0-indexed tree-sitter position to 1-indexed line number
        let node_line = node.start_position().row as u32 + 1;

        // Skip nodes outside our range
        if node_line < start_line || node_line > end_line {
            return;
        }

        let kind = node.kind();

        // Skip ignored nodes
        if self.ignored_nodes.contains(kind) {
            return;
        }

        // Check if this is a decision node
        if self.decision_nodes.contains(kind) {
            if kind == "binary_expression" || kind == "boolean_operator" {
                if self.is_logical_operator(node) {
                    *cc += 1;
                }
            } else {
                *cc += 1;
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cyclomatic_range(child, start_line, end_line, cc);
        }
    }

    /// Check if a binary expression is a logical operator (&& or ||).
    fn is_logical_operator(&self, node: Node) -> bool {
        // Look for operator child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if kind == "&&" || kind == "||" || kind == "and" || kind == "or" {
                return true;
            }
        }
        false
    }

    // ========================================================================
    // Cognitive Complexity
    // ========================================================================

    /// Compute cognitive complexity for a node.
    ///
    /// Cognitive complexity adds a nesting penalty for each level of nesting.
    pub fn compute_cognitive(&self, node: Node, nesting_level: u32) -> u32 {
        let mut cog = 0;

        self.visit_for_cognitive(node, nesting_level, &mut cog);

        cog
    }

    /// Compute cognitive complexity within a line range.
    fn compute_cognitive_range(
        &self,
        node: Node,
        start_line: u32,
        end_line: u32,
        nesting_level: u32,
    ) -> u32 {
        let mut cog = 0;

        self.visit_for_cognitive_range(node, start_line, end_line, nesting_level, &mut cog);

        cog
    }

    fn visit_for_cognitive(&self, node: Node, nesting_level: u32, cog: &mut u32) {
        let kind = node.kind();

        // Skip ignored nodes
        if self.ignored_nodes.contains(kind) {
            return;
        }

        let is_nesting = self.nesting_nodes.contains(kind);
        let is_decision = self.decision_nodes.contains(kind);

        // Decision nodes add base + nesting penalty
        if is_decision {
            if kind == "binary_expression" || kind == "boolean_operator" {
                if self.is_logical_operator(node) {
                    *cog += 1; // Logical operators don't get nesting penalty
                }
            } else {
                *cog += 1 + nesting_level; // Base + nesting penalty
            }
        }

        // Recurse with updated nesting level
        let new_nesting = if is_nesting {
            nesting_level + 1
        } else {
            nesting_level
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cognitive(child, new_nesting, cog);
        }
    }

    fn visit_for_cognitive_range(
        &self,
        node: Node,
        start_line: u32,
        end_line: u32,
        nesting_level: u32,
        cog: &mut u32,
    ) {
        // Convert 0-indexed tree-sitter position to 1-indexed line number
        let node_line = node.start_position().row as u32 + 1;

        // Skip nodes outside our range
        if node_line < start_line || node_line > end_line {
            return;
        }

        let kind = node.kind();

        // Skip ignored nodes
        if self.ignored_nodes.contains(kind) {
            return;
        }

        let is_nesting = self.nesting_nodes.contains(kind);
        let is_decision = self.decision_nodes.contains(kind);

        if is_decision {
            if kind == "binary_expression" || kind == "boolean_operator" {
                if self.is_logical_operator(node) {
                    *cog += 1;
                }
            } else {
                *cog += 1 + nesting_level;
            }
        }

        let new_nesting = if is_nesting {
            nesting_level + 1
        } else {
            nesting_level
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cognitive_range(child, start_line, end_line, new_nesting, cog);
        }
    }

    // ========================================================================
    // Halstead Metrics
    // ========================================================================

    /// Compute Halstead metrics for a code range.
    pub fn compute_halstead(
        &self,
        node: Node,
        source: &[u8],
        start_line: u32,
        end_line: u32,
    ) -> HalsteadMetrics {
        let mut operators: HashMap<String, u32> = HashMap::new();
        let mut operands: HashMap<String, u32> = HashMap::new();

        self.collect_halstead(
            node,
            source,
            start_line,
            end_line,
            &mut operators,
            &mut operands,
        );

        // Calculate derived metrics
        let n1 = operators.len() as u32; // Distinct operators
        let n2 = operands.len() as u32; // Distinct operands
        let big_n1: u32 = operators.values().sum(); // Total operators
        let big_n2: u32 = operands.values().sum(); // Total operands

        HalsteadMetrics::calculate(n1, n2, big_n1, big_n2)
    }

    fn collect_halstead(
        &self,
        node: Node,
        source: &[u8],
        start_line: u32,
        end_line: u32,
        operators: &mut HashMap<String, u32>,
        operands: &mut HashMap<String, u32>,
    ) {
        // Convert 0-indexed tree-sitter position to 1-indexed line number
        let node_line = node.start_position().row as u32 + 1;

        // Skip nodes outside our range
        if node_line < start_line || node_line > end_line {
            // But still check children - they might be in range
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.collect_halstead(child, source, start_line, end_line, operators, operands);
            }
            return;
        }

        let kind = node.kind();
        let text = node.utf8_text(source).unwrap_or("").to_string();

        // Classify node as operator or operand
        if self.is_operator_node(kind) {
            *operators.entry(text).or_insert(0) += 1;
        } else if self.is_operand_node(kind) {
            // Track operand text (identifiers, literals)
            if !text.is_empty() && text.len() < 100 {
                // Avoid huge literals
                *operands.entry(text).or_insert(0) += 1;
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_halstead(child, source, start_line, end_line, operators, operands);
        }
    }

    /// Check if a node type is an operator.
    fn is_operator_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            // Arithmetic
            "+"
                | "-"
                | "*"
                | "/"
                | "%"
                | "**"
                // Comparison
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                // Logical
                | "&&"
                | "||"
                | "!"
                | "and"
                | "or"
                | "not"
                // Assignment
                | "="
                | "+="
                | "-="
                | "*="
                | "/="
                // Bitwise
                | "&"
                | "|"
                | "^"
                | "~"
                | "<<"
                | ">>"
                // Access
                | "."
                | "::"
                | "->"
                | "?."
                // Other
                | "?"
                | ":"
                | "=>"
                | ".."
                | "..."
                // Keywords as operators
                | "if"
                | "else"
                | "while"
                | "for"
                | "loop"
                | "match"
                | "return"
                | "break"
                | "continue"
                | "let"
                | "const"
                | "var"
                | "def"
                | "fn"
                | "async"
                | "await"
                | "try"
                | "catch"
                | "except"
                | "finally"
                | "raise"
                | "throw"
                | "yield"
                | "import"
                | "from"
                | "use"
        )
    }

    /// Check if a node type is an operand.
    fn is_operand_node(&self, kind: &str) -> bool {
        matches!(
            kind,
            "identifier"
                | "field_identifier"
                | "property_identifier"
                | "type_identifier"
                | "integer"
                | "integer_literal"
                | "float"
                | "float_literal"
                | "string"
                | "string_literal"
                | "raw_string_literal"
                | "char_literal"
                | "boolean"
                | "true"
                | "false"
                | "none"
                | "null"
                | "nil"
        )
    }

    // ========================================================================
    // Line Counting
    // ========================================================================

    /// Count logical lines, total lines, and comment lines in a range.
    fn count_lines(&self, source: &[u8], start_line: u32, end_line: u32) -> (u32, u32, u32) {
        let source_str = String::from_utf8_lossy(source);
        let lines: Vec<&str> = source_str.lines().collect();

        let start = start_line.saturating_sub(1) as usize; // Convert to 0-indexed
        let end = (end_line as usize).min(lines.len());

        if start >= lines.len() {
            return (0, 0, 0);
        }

        let range_lines = &lines[start..end];

        let total = range_lines.len() as u32;
        let mut logical = 0u32;
        let mut comments = 0u32;
        let mut in_block_comment = false;

        for line in range_lines {
            let trimmed = line.trim();

            // Detect block comment markers
            let is_block_start = trimmed.starts_with("/*") || trimmed.starts_with("\"\"\"");
            let is_block_end = trimmed.ends_with("*/") || trimmed.ends_with("\"\"\"");

            // Enter multi-line block comment only if it doesn't end on same line
            if is_block_start && !is_block_end {
                in_block_comment = true;
            }

            // Count as comment if in block, single-line block, or line comment
            if in_block_comment
                || (is_block_start && is_block_end)
                || trimmed.starts_with("//")
                || trimmed.starts_with("#")
            {
                comments += 1;
            } else if !trimmed.is_empty() {
                logical += 1;
            }

            // End of multi-line block comment
            if is_block_end {
                in_block_comment = false;
            }
        }

        (logical, total, comments)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper: create a mock grammar for testing
    struct TestGrammar;

    impl Grammar for TestGrammar {
        fn language_id(&self) -> &'static str {
            "test"
        }

        fn file_extensions(&self) -> &'static [&'static str] {
            &[".test"]
        }

        fn ts_language(&self) -> tree_sitter::Language {
            // We can't actually return a real language without a grammar
            // These tests are for the logic, not parsing
            panic!("TestGrammar doesn't support parsing")
        }

        fn function_query(&self) -> &str {
            ""
        }

        fn call_query(&self) -> &str {
            ""
        }

        fn import_query(&self) -> &str {
            ""
        }

        fn decision_nodes(&self) -> &'static [&'static str] {
            &["if_statement", "for_statement", "while_statement"]
        }

        fn nesting_nodes(&self) -> &'static [&'static str] {
            &["if_statement", "for_statement"]
        }

        fn compute_module_path(
            &self,
            _file_path: &std::path::Path,
            _root: &std::path::Path,
        ) -> String {
            "test::module".to_string()
        }
    }

    #[test]
    fn test_calculator_creation() {
        let grammar = TestGrammar;
        let calc = ComplexityCalculator::new(&grammar);

        assert!(calc.decision_nodes.contains("if_statement"));
        assert!(calc.nesting_nodes.contains("for_statement"));
    }

    #[test]
    fn test_halstead_metrics_calculation() {
        // Test the HalsteadMetrics::calculate function
        let metrics = HalsteadMetrics::calculate(10, 20, 50, 100);

        assert_eq!(metrics.vocabulary, 30);
        assert_eq!(metrics.length, 150);
        assert!(metrics.volume > 0.0);
        assert!(metrics.difficulty > 0.0);
        assert!(metrics.effort > 0.0);
    }

    #[test]
    fn test_halstead_zero_division() {
        // Edge case: no operators or operands
        let metrics = HalsteadMetrics::calculate(0, 0, 0, 0);

        assert_eq!(metrics.vocabulary, 0);
        assert_eq!(metrics.length, 0);
        assert_eq!(metrics.volume, 0.0);
        assert_eq!(metrics.difficulty, 0.0);
    }

    #[test]
    fn test_line_counting() {
        let grammar = TestGrammar;
        let calc = ComplexityCalculator::new(&grammar);

        let source = b"fn main() {\n    // comment\n    let x = 1;\n}\n";
        let (logical, total, comments) = calc.count_lines(source, 1, 4);

        assert_eq!(total, 4);
        assert_eq!(comments, 1);
        assert_eq!(logical, 3); // fn main, let x, and closing brace
    }

    #[test]
    fn test_is_operator_node() {
        let grammar = TestGrammar;
        let calc = ComplexityCalculator::new(&grammar);

        assert!(calc.is_operator_node("+"));
        assert!(calc.is_operator_node("=="));
        assert!(calc.is_operator_node("if"));
        assert!(calc.is_operator_node("return"));
        assert!(!calc.is_operator_node("identifier"));
    }

    #[test]
    fn test_is_operand_node() {
        let grammar = TestGrammar;
        let calc = ComplexityCalculator::new(&grammar);

        assert!(calc.is_operand_node("identifier"));
        assert!(calc.is_operand_node("integer_literal"));
        assert!(calc.is_operand_node("string_literal"));
        assert!(!calc.is_operand_node("+"));
    }
}
