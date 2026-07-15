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
    /// CYCLOMATIC (McCabe) decision nodes: each `switch_case`/`match_arm` counts.
    cyclomatic_decision_nodes: HashSet<&'static str>,
    /// COGNITIVE (SonarSource) decision nodes: a `switch`/`match` structure
    /// counts once instead of once per branch.
    cognitive_decision_nodes: HashSet<&'static str>,
    nesting_nodes: HashSet<&'static str>,
    ignored_nodes: HashSet<&'static str>,
}

impl<'g> ComplexityCalculator<'g> {
    /// Create a new calculator for the given grammar.
    pub fn new(grammar: &'g dyn Grammar) -> Self {
        Self {
            grammar,
            cyclomatic_decision_nodes: grammar.decision_nodes().iter().copied().collect(),
            cognitive_decision_nodes: grammar.cognitive_decision_nodes().iter().copied().collect(),
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

    /// Find the SINGLE outermost function-like node fully ENCLOSED by
    /// `[start, end]`, or `None` if the range encloses zero or several sibling
    /// functions.
    ///
    /// Complements `find_node_in_range` (which finds a function CONTAINING the
    /// range). Used by the fallback so a function whose enclosing construct
    /// (class body, export wrapper) is the outer node is still measured at
    /// nesting level 0 rather than descended-through and inflated (finding 1).
    ///
    /// We only delegate to `compute_cognitive` when EXACTLY ONE function is
    /// enclosed. When the range encloses multiple sibling top-level functions,
    /// returning the first preorder match would silently measure just that one
    /// function and ignore the rest. In that case we return `None` so the caller
    /// retains the range-traversal path and accounts for every enclosed function.
    fn find_enclosed_function<'a>(
        &self,
        node: Node<'a>,
        start_line: u32,
        end_line: u32,
    ) -> Option<Node<'a>> {
        let mut found = Vec::new();
        self.collect_enclosed_functions(node, start_line, end_line, &mut found);
        match found.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }

    /// Collect the outermost function-like nodes fully ENCLOSED by
    /// `[start, end]` without descending into a function once one is found (so
    /// nested functions are not double-collected).
    fn collect_enclosed_functions<'a>(
        &self,
        node: Node<'a>,
        start_line: u32,
        end_line: u32,
        out: &mut Vec<Node<'a>>,
    ) {
        let node_start = node.start_position().row as u32 + 1;
        let node_end = node.end_position().row as u32 + 1;

        if node_start >= start_line && node_end <= end_line && self.is_function_node(node.kind()) {
            out.push(node);
            return;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_enclosed_functions(child, start_line, end_line, out);
        }
    }

    /// Check if a node type represents a function.
    fn is_function_node(&self, kind: &str) -> bool {
        // Common function node types across languages.
        //
        // `function_expression` (`const f = function () { ... }`) and the
        // generator variants are extracted by TS_FUNCTION_QUERY, so they MUST be
        // recognized here too. Otherwise `compute_metrics` misses the function
        // node and falls back to the range path, which descends through the
        // function node itself (a nesting node) and inflates cognitive
        // complexity by one nesting level. Keeping this list consistent with the
        // function extractor guarantees every measured function uses the
        // corrected `compute_cognitive` path.
        matches!(
            kind,
            "function_item"
                | "function_definition"
                | "function_declaration"
                | "function_expression"
                | "generator_function"
                | "generator_function_declaration"
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
        if self.cyclomatic_decision_nodes.contains(kind) {
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
        if self.cyclomatic_decision_nodes.contains(kind) {
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
    // Cognitive Complexity (SonarSource reference algorithm)
    //
    // Three ingredients from the SonarSource paper
    // (https://www.sonarsource.com/docs/CognitiveComplexity.pdf):
    //
    // - B1 (increment +1): each `if`, `else if`, `else`, ternary, `switch`,
    //   loop, `catch`, and each SEQUENCE of like logical operators (`&&`/`||`).
    // - B2 (nesting): `if`, `else`/`else if`, ternary, `switch`, loop, `catch`,
    //   and nested functions/lambdas raise the nesting level of their body.
    // - B3 (nesting penalty): a structure that BOTH increments AND nests
    //   (`if`, ternary, `switch`, loop, `catch`) adds the current nesting level
    //   on top of its +1. `else`/`else if`, logical operators, and recursion
    //   increment but take NO penalty.
    //
    // The measured function's own node is nesting level 0: `compute_cognitive`
    // visits its CHILDREN at the incoming level, so the function definition is
    // never itself counted as a nesting level.
    // ========================================================================

    /// Compute cognitive complexity for a function node (visited at level 0).
    ///
    /// `node` is the measured function's own tree-sitter node. Its CHILDREN are
    /// visited at the incoming `nesting_level`, so the function's own definition
    /// does not count as a nesting level. Genuinely-nested functions/closures
    /// encountered while descending the body ARE in `nesting_nodes`, so they
    /// correctly raise the nesting level. Matches the SonarSource reference.
    pub fn compute_cognitive(&self, node: Node, nesting_level: u32) -> u32 {
        let mut cog = 0;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cognitive(child, nesting_level, None, &mut cog);
        }

        cog
    }

    /// Compute cognitive complexity within a line range (fallback path).
    ///
    /// Finding 1: if a function-like node bounds this range, delegate to
    /// `compute_cognitive` so the function's own node stays at nesting level 0.
    /// Visiting the range's raw children would descend THROUGH the function node
    /// (a nesting node) and inflate every construct in its body by one level.
    /// Only when no function node bounds the range (module-level code) do we
    /// visit children directly.
    fn compute_cognitive_range(
        &self,
        node: Node,
        start_line: u32,
        end_line: u32,
        nesting_level: u32,
    ) -> u32 {
        // Prefer a function-like node that CONTAINS the range; failing that, one
        // ENCLOSED by the range (e.g. a method whose class is the outer node, or
        // an export/const wrapper measured with the whole file's range). Either
        // way we measure it at nesting level 0 via `compute_cognitive`.
        if let Some(function_node) = self
            .find_node_in_range(node, start_line, end_line)
            .or_else(|| self.find_enclosed_function(node, start_line, end_line))
        {
            return self.compute_cognitive(function_node, nesting_level);
        }

        let mut cog = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cognitive(child, nesting_level, Some((start_line, end_line)), &mut cog);
        }
        cog
    }

    /// Core cognitive-complexity visitor.
    ///
    /// `range` optionally restricts counting to nodes whose start line is inside
    /// `[start, end]` (used only by the module-level fallback path).
    fn visit_for_cognitive(
        &self,
        node: Node,
        nesting_level: u32,
        range: Option<(u32, u32)>,
        cog: &mut u32,
    ) {
        if let Some((start_line, end_line)) = range {
            let node_line = node.start_position().row as u32 + 1;
            if node_line < start_line || node_line > end_line {
                return;
            }
        }

        let kind = node.kind();

        // Skip ignored nodes (e.g. `?.`, `?`, `try`-`?` operators).
        if self.ignored_nodes.contains(kind) {
            return;
        }

        // Logical operator sequences (B1). SonarSource charges +1 per RUN of the
        // same operator, plus +1 whenever the operator changes within a chain,
        // and NO nesting penalty. tree-sitter left-associates `a && b && c` as
        // `((a && b) && c)`, so we charge a logical node only when it STARTS a
        // new run: i.e. when its parent is not a logical binary with the same
        // operator (finding 5). The nesting level is unchanged by a logical op.
        if (kind == "binary_expression" || kind == "boolean_operator")
            && self.cognitive_decision_nodes.contains(kind)
        {
            if let Some(op) = self.logical_operator_of(node) {
                if !self.parent_is_same_logical_operator(node, op) {
                    *cog += 1;
                }
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.visit_for_cognitive(child, nesting_level, range, cog);
            }
            return;
        }

        let (increment, penalty, bump) = self.cognitive_effect(node);

        if increment {
            *cog += 1 + if penalty { nesting_level } else { 0 };
        }

        let new_nesting = if bump {
            nesting_level + 1
        } else {
            nesting_level
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_cognitive(child, new_nesting, range, cog);
        }
    }

    /// SonarSource classification for a single node.
    ///
    /// Returns `(increment, nesting_penalty, increases_nesting)`:
    /// - `increment`: adds +1 (B1).
    /// - `nesting_penalty`: the +1 also carries the current nesting level (B3).
    /// - `increases_nesting`: the body is one level deeper (B2).
    ///
    /// Logical operators are handled by the caller (run detection), so they are
    /// reported as no-ops here.
    fn cognitive_effect(&self, node: Node) -> (bool, bool, bool) {
        let kind = node.kind();

        if kind == "binary_expression" || kind == "boolean_operator" {
            return (false, false, false);
        }

        // `break`/`continue` to a LABEL is a B1 increment (+1, no nesting
        // penalty, no nesting). An unlabeled break/continue is not counted. A
        // labeled form carries a label identifier child (`statement_identifier`
        // in TS/JS, `label` in Rust).
        if kind == "break_statement"
            || kind == "continue_statement"
            || kind == "break_expression"
            || kind == "continue_expression"
        {
            let mut cursor = node.walk();
            let has_label = node
                .children(&mut cursor)
                .any(|c| c.kind() == "statement_identifier" || c.kind() == "label");
            return (has_label, false, false);
        }

        // `else if`: TS/Rust model it as an `if` nested inside an `else_clause`.
        // It increments +1 with NO nesting penalty and does NOT add a nesting
        // level (its body sits at the same level as the original `if` body).
        // The wrapping `else_clause` must NOT also increment (finding 4).
        if self.is_else_if(node) {
            return (true, false, false);
        }

        // `else_clause`: a plain `else` (wrapping a block) increments +1 with no
        // penalty. An `else_clause` wrapping an `if` (the "else if" case) does
        // NOT increment here; the inner `if` carries the single +1. Either way
        // it does not raise nesting: the enclosing `if` already did (finding 4).
        if kind == "else_clause" {
            let wraps_if = self.node_has_if_child(node);
            return (!wraps_if, false, false);
        }

        // General structures. A node that both increments and nests (if,
        // ternary, switch/match, loops, catch/except) takes the B3 penalty; a
        // pure nesting node (function/lambda/closure/with) only raises nesting.
        let is_decision = self.cognitive_decision_nodes.contains(kind);
        let is_nesting = self.nesting_nodes.contains(kind);
        let increment = is_decision;
        let penalty = is_decision && is_nesting;
        (increment, penalty, is_nesting)
    }

    /// Whether `node` is an `else if`: an `if` node whose parent is an
    /// `else_clause` (TypeScript and Rust structure).
    fn is_else_if(&self, node: Node) -> bool {
        if node.kind() != "if_statement" && node.kind() != "if_expression" {
            return false;
        }
        node.parent().map(|p| p.kind()) == Some("else_clause")
    }

    /// Whether `node` has a direct child that is an `if` (used to distinguish an
    /// `else if` `else_clause` from a plain `else`).
    fn node_has_if_child(&self, node: Node) -> bool {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .any(|c| c.kind() == "if_statement" || c.kind() == "if_expression")
    }

    /// Return the logical operator token of a logical binary node, if any.
    fn logical_operator_of(&self, node: Node) -> Option<&'static str> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "&&" => return Some("&&"),
                "||" => return Some("||"),
                "and" => return Some("and"),
                "or" => return Some("or"),
                _ => {}
            }
        }
        None
    }

    /// Whether `node`'s effective logical parent is a logical binary using the
    /// same operator. Used to charge only the FIRST node of each same-operator
    /// run (finding 5).
    ///
    /// Parentheses are TRANSPARENT for operator continuation: SonarSource charges
    /// +1 per RUN of the same operator, and a parenthesized subexpression does
    /// NOT split a run. tree-sitter models `a && (b && c)` so the inner `&&`'s
    /// direct parent is a `parenthesized_expression`, not the outer
    /// `binary_expression`/`boolean_operator`. We therefore walk UP through any
    /// `parenthesized_expression` ancestors before comparing the operator token;
    /// otherwise a parenthesized same-operator subexpression is mistaken for a
    /// fresh run and over-counted by one.
    fn parent_is_same_logical_operator(&self, node: Node, op: &str) -> bool {
        let mut current = node;
        while let Some(parent) = current.parent() {
            match parent.kind() {
                "parenthesized_expression" => {
                    current = parent;
                }
                "binary_expression" | "boolean_operator" => {
                    return self.logical_operator_of(parent) == Some(op);
                }
                _ => return false,
            }
        }
        false
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

        assert!(calc.cyclomatic_decision_nodes.contains("if_statement"));
        assert!(calc.cognitive_decision_nodes.contains("if_statement"));
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

    // ========================================================================
    // Cognitive Complexity: SonarSource reference-value tests
    //
    // These parse real source with the concrete tree-sitter grammars and
    // assert the exact values from the SonarSource Cognitive Complexity
    // specification (https://www.sonarsource.com/docs/CognitiveComplexity.pdf).
    //
    // Regression guard: before the "off-by-one nesting" fix, the measured
    // function's OWN node was counted as a nesting level, so `flat_ifs`
    // returned 2N instead of N and `nested_ifs_depth_3` returned 9 instead of
    // 6. The `nested_function` case proves that genuinely-nested
    // functions/closures STILL add a nesting level after the fix.
    // ========================================================================

    use crate::adapter::grammars::{PythonGrammar, RustGrammar, TypeScriptGrammar};

    /// Parse `source` with `grammar`, locate the (first/only) function that
    /// spans the whole snippet, and return its cognitive complexity.
    fn cognitive_of(grammar: &dyn Grammar, source: &str) -> u32 {
        metrics_of(grammar, source).cognitive_complexity
    }

    /// Parse `source` with `grammar`, locate the function spanning the snippet,
    /// and return its cyclomatic complexity.
    fn cyclomatic_of(grammar: &dyn Grammar, source: &str) -> u32 {
        metrics_of(grammar, source).cyclomatic_complexity
    }

    /// Parse `source` with `grammar` and compute metrics for the whole snippet.
    fn metrics_of(grammar: &dyn Grammar, source: &str) -> FunctionMetrics {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&grammar.ts_language())
            .expect("set language");
        let tree = parser.parse(source, None).expect("parse");
        let calc = ComplexityCalculator::new(grammar);
        let end_line = source.lines().count() as u32;
        calc.compute_metrics(&tree, source.as_bytes(), 1, end_line)
    }

    #[test]
    fn ts_cognitive_flat_ifs_equals_n() {
        let grammar = TypeScriptGrammar::new();
        // N flat, sibling ifs → cognitive == N (each +1, no nesting).
        let src = "function flat4(a, b, c, d) {\n  if (a) { return; }\n  if (b) { return; }\n  if (c) { return; }\n  if (d) { return; }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 4);
    }

    #[test]
    fn ts_cognitive_nested_ifs_depth_3_equals_6() {
        let grammar = TypeScriptGrammar::new();
        // 3 nested ifs → (1+0) + (1+1) + (1+2) == 6.
        let src = "function nested3(a) {\n  if (a) {\n    if (a) {\n      if (a) { return; }\n    }\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 6);
    }

    #[test]
    fn ts_cognitive_nested_function_adds_nesting() {
        let grammar = TypeScriptGrammar::new();
        // The inner arrow function nests the `if` one level deeper:
        // the if is charged 1 + 1 == 2. If nested functions did NOT add a
        // nesting level, this would be 1, so this proves the fix preserves
        // real nesting from inner functions/closures.
        let src = "function outer(a) {\n  const inner = (b) => {\n    if (b) { return; }\n  };\n  inner(a);\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 2);
    }

    #[test]
    fn ts_cognitive_switch_counted_once_not_per_case() {
        let grammar = TypeScriptGrammar::new();
        // A switch with 3 cases → +1 for the switch structure only (nesting 0),
        // NOT +1 per case. SonarSource treats a switch as a single structural
        // increment.
        let src = "function sw(a) {\n  switch (a) {\n    case 1: return;\n    case 2: return;\n    default: return;\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 1);
    }

    #[test]
    fn rust_cognitive_flat_ifs_equals_n() {
        let grammar = RustGrammar::new();
        // Rust's nesting set excludes `function_item`, so the top-level path was
        // already correct; this asserts the fix is a no-op for Rust.
        let src = "fn flat2(a: bool) {\n    if a { return; }\n    if a { return; }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 2);
    }

    #[test]
    fn rust_cognitive_nested_ifs_depth_3_equals_6() {
        let grammar = RustGrammar::new();
        let src = "fn nested3(a: bool) {\n    if a {\n        if a {\n            if a { return; }\n        }\n    }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 6);
    }

    #[test]
    fn rust_cognitive_match_counted_once_not_per_arm() {
        let grammar = RustGrammar::new();
        // A match with 3 arms → +1 for the match structure only, not per arm.
        let src = "fn m(a: i32) -> i32 {\n    match a {\n        1 => 1,\n        2 => 2,\n        _ => 0,\n    }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 1);
    }

    // ========================================================================
    // Metrics divergence: switch/match must count per case/arm for CYCLOMATIC
    // (McCabe) but once for COGNITIVE (SonarSource). These paired tests guard
    // the regression where sharing one decision set made cyclomatic under-count
    // switches/matches after the cognitive fix.
    // ========================================================================

    #[test]
    fn ts_cyclomatic_switch_counts_each_case() {
        let grammar = TypeScriptGrammar::new();
        // switch with `case 1`, `case 2`, `default` → two `switch_case` nodes
        // (default is a `switch_default`), so McCabe == 1 (base) + 2 == 3.
        let src = "function sw(a) {\n  switch (a) {\n    case 1: return;\n    case 2: return;\n    default: return;\n  }\n}\n";
        assert_eq!(cyclomatic_of(&grammar, src), 3);
        // Same snippet, cognitive charges the switch structure once.
        assert_eq!(cognitive_of(&grammar, src), 1);
    }

    #[test]
    fn rust_cyclomatic_match_counts_each_arm() {
        let grammar = RustGrammar::new();
        // match with 3 arms (including the wildcard `_`) → three `match_arm`
        // nodes, so McCabe == 1 (base) + 3 == 4.
        let src = "fn m(a: i32) -> i32 {\n    match a {\n        1 => 1,\n        2 => 2,\n        _ => 0,\n    }\n}\n";
        assert_eq!(cyclomatic_of(&grammar, src), 4);
        // Same snippet, cognitive charges the match structure once.
        assert_eq!(cognitive_of(&grammar, src), 1);
    }

    #[test]
    fn ts_cognitive_function_expression_not_off_by_one() {
        let grammar = TypeScriptGrammar::new();
        // `const f = function () { if (a) { if (b) {} } }`.
        // The outer if is at nesting 0 (+1), the inner if at nesting 1 (+2),
        // so cognitive == 3, identical to the equivalent function_declaration.
        //
        // Before recognizing `function_expression` in `is_function_node`, this
        // fell back to the range path, which descends through the
        // function_expression node itself (a nesting node) and inflated the
        // result to 5. This asserts the corrected `compute_cognitive` path runs.
        let src = "const f = function () {\n  if (a) {\n    if (b) {\n    }\n  }\n};\n";
        assert_eq!(cognitive_of(&grammar, src), 3);

        // Equivalence check: the same body as a function_declaration.
        let decl = "function f() {\n  if (a) {\n    if (b) {\n    }\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, decl), 3);
    }

    // ========================================================================
    // SonarSource panel findings 1-6: each has a reference-value micro-test
    // that FAILS before its fix and PASSES after. Values are taken directly
    // from the SonarSource Cognitive Complexity specification.
    // ========================================================================

    // --- Finding 1: range-fallback off-by-one + routing ------------------

    #[test]
    fn ts_cognitive_range_fallback_not_off_by_one() {
        // Directly exercise the fallback path `compute_cognitive_range`. Before
        // the fix it visited `root`'s children, descending THROUGH the
        // `function_declaration` (a nesting node) and charging the whole body one
        // level too deep: (1+1) + (1+2) == 5. The fix delegates to
        // `compute_cognitive` on the located function node (nesting 0), so the
        // outer if is +1 and the inner if is +1+1 == 3.
        let grammar = TypeScriptGrammar::new();
        let src = "function f() {\n  if (a) {\n    if (b) {\n    }\n  }\n}\n";
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar.ts_language()).unwrap();
        let tree = parser.parse(src, None).unwrap();
        let calc = ComplexityCalculator::new(&grammar);
        let end = src.lines().count() as u32;
        assert_eq!(calc.compute_cognitive_range(tree.root_node(), 1, end, 0), 3);
    }

    #[test]
    fn ts_cognitive_every_function_form_equals_bare_declaration() {
        // Every function form (bare declaration, export-wrapped, const-assigned
        // arrow, method) must measure the SAME body as nesting level 0, yielding
        // 3 for `if (a) { if (b) {} }`, never the inflated 5 from routing a
        // wrapper node through the range fallback.
        let grammar = TypeScriptGrammar::new();
        let bare = "function f() {\n  if (a) {\n    if (b) {\n    }\n  }\n}\n";
        let exported = "export function f() {\n  if (a) {\n    if (b) {\n    }\n  }\n}\n";
        let arrow = "const f = () => {\n  if (a) {\n    if (b) {\n    }\n  }\n};\n";
        let method = "class C {\n  m() {\n    if (a) {\n      if (b) {\n      }\n    }\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, bare), 3);
        assert_eq!(cognitive_of(&grammar, exported), 3);
        assert_eq!(cognitive_of(&grammar, arrow), 3);
        assert_eq!(cognitive_of(&grammar, method), 3);
    }

    #[test]
    fn ts_cognitive_range_with_multiple_sibling_functions_measures_all() {
        // When a range encloses several sibling top-level functions, the fallback
        // must NOT collapse to just the first function's value. Before the fix,
        // `find_enclosed_function` returned the first preorder function, so only
        // `a`'s single decision was measured (cognitive == 1). The fix returns
        // `None` for a multi-function range so the range-traversal path accounts
        // for both functions' decisions, yielding a value greater than 1.
        let grammar = TypeScriptGrammar::new();
        let src = "function a() {\n  if (x) {\n  }\n}\nfunction b() {\n  if (y) {\n  }\n}\n";
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar.ts_language()).unwrap();
        let tree = parser.parse(src, None).unwrap();
        let calc = ComplexityCalculator::new(&grammar);
        let end = src.lines().count() as u32;
        let cognitive = calc.compute_cognitive_range(tree.root_node(), 1, end, 0);
        assert!(
            cognitive > 1,
            "multi-function range must not collapse to the first function's value of 1, got {cognitive}"
        );
    }

    // --- Finding 2: try does not nest; catch increments and nests --------

    #[test]
    fn ts_cognitive_try_catch_not_inverted() {
        let grammar = TypeScriptGrammar::new();
        // `try {} catch(e) {}`: the try adds nothing, the catch is a single +1.
        let simple = "function f() {\n  try {\n  } catch (e) {\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, simple), 1);
        // `try {} catch(e){ if(e){} }`: catch +1 (nesting 0) then if +1+1 == 3.
        // Before the fix `try_statement` nested (so the if was one level too
        // deep) while `catch_clause` did not, inverting the SonarSource rule.
        let nested = "function f() {\n  try {\n  } catch (e) {\n    if (e) {\n    }\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, nested), 3);
    }

    // --- Finding 3: ternary both increments and nests --------------------

    #[test]
    fn ts_cognitive_ternary_nests() {
        let grammar = TypeScriptGrammar::new();
        // `a ? (b ? c : d) : e`: outer ternary +1 (nesting 0), inner ternary
        // +1+1 == 3. Before adding `ternary_expression` to the nesting set the
        // inner ternary took no penalty and the total was 2.
        let src = "function f() {\n  return a ? (b ? c : d) : e;\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 3);
    }

    // --- Finding 4: else / else-if counted once, no penalty --------------

    #[test]
    fn ts_cognitive_if_elseif_else_each_plus_one() {
        let grammar = TypeScriptGrammar::new();
        // if +1, else-if +1, else +1 == 3. No nesting penalties, and the
        // `else_clause` wrapping the else-if must not double-count.
        let src = "function f() {\n  if (a) {\n  } else if (b) {\n  } else {\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 3);
    }

    #[test]
    fn ts_cognitive_elseif_body_nests_but_elseif_itself_does_not() {
        let grammar = TypeScriptGrammar::new();
        // if(a){ if(x){} } else if(b){ if(y){} }:
        //   outer if 1, inner x (1+1)=2, else-if 1, inner y (1+1)=2 == 6.
        // The else-if increments +1 with no penalty and does NOT add a nesting
        // level, so its own body's inner if sits at nesting 1, not 2.
        let src = "function f() {\n  if (a) {\n    if (x) {\n    }\n  } else if (b) {\n    if (y) {\n    }\n  }\n}\n";
        assert_eq!(cognitive_of(&grammar, src), 6);
    }

    // --- Finding 5: logical operator sequences -------------------------

    #[test]
    fn ts_cognitive_logical_sequences_counted_per_run() {
        let grammar = TypeScriptGrammar::new();
        // if + one && run == 2 (before: if + two && nodes == 3).
        assert_eq!(
            cognitive_of(&grammar, "function f() {\n  if (a && b && c) {\n  }\n}\n"),
            2
        );
        // if + && run + || run == 3.
        assert_eq!(
            cognitive_of(&grammar, "function f() {\n  if (a && b || c) {\n  }\n}\n"),
            3
        );
        // if + || run + && run == 3 (&& binds tighter, forming a separate run).
        assert_eq!(
            cognitive_of(
                &grammar,
                "function f() {\n  if (a || b || c && d) {\n  }\n}\n"
            ),
            3
        );
    }

    // --- Parenthesized same-operator runs are transparent ----------------
    //
    // SonarSource charges +1 per RUN of the same logical operator, and
    // parentheses do NOT split a run. Before walking up through
    // `parenthesized_expression` ancestors, the inner logical node's direct
    // parent was the parens (not a logical binary), so it was mistaken for a
    // fresh run and over-counted by one.

    #[test]
    fn ts_cognitive_parenthesized_same_operator_run_not_overcounted() {
        let grammar = TypeScriptGrammar::new();
        // if (a && (b && c)): one `&&` run (+1) plus the `if` (+1) == 2.
        // Before the fix this returned 3 (the parenthesized inner `&&` counted
        // as a second run).
        assert_eq!(
            cognitive_of(&grammar, "function f() {\n  if (a && (b && c)) {\n  }\n}\n"),
            2
        );
        // if (a && (b || c) && d): outer `&&` run (+1), the parenthesized `||`
        // differs from its `&&` parent (+1), plus the `if` (+1) == 3. This value
        // must be unchanged by the fix (it was already 3, not 4).
        assert_eq!(
            cognitive_of(
                &grammar,
                "function f() {\n  if (a && (b || c) && d) {\n  }\n}\n"
            ),
            3
        );
        // Unparenthesized regressions must stay put: `&&` + `||` + `if` == 3.
        assert_eq!(
            cognitive_of(&grammar, "function f() {\n  if (a && b || c) {\n  }\n}\n"),
            3
        );
        // and a single `&&` run + `if` == 2.
        assert_eq!(
            cognitive_of(&grammar, "function f() {\n  if (a && b && c) {\n  }\n}\n"),
            2
        );
    }

    #[test]
    fn python_cognitive_parenthesized_same_operator_run_not_overcounted() {
        let grammar = PythonGrammar::new();
        // if a and (b and c): one `and` run (+1) plus the `if` (+1) == 2.
        // Before the fix this returned 3.
        assert_eq!(
            cognitive_of(
                &grammar,
                "def f(a, b, c):\n    if a and (b and c):\n        pass\n"
            ),
            2
        );
        // if a and (b or c) and d: `and` run (+1), parenthesized `or` differs
        // (+1), plus `if` (+1) == 3.
        assert_eq!(
            cognitive_of(
                &grammar,
                "def f(a, b, c, d):\n    if a and (b or c) and d:\n        pass\n"
            ),
            3
        );
    }

    // --- Finding 6: Python match ----------------------------------------

    #[test]
    fn python_match_cyclomatic_per_case_cognitive_once() {
        let grammar = PythonGrammar::new();
        let src = "def f(x):\n    match x:\n        case 1:\n            return 1\n        case _:\n            return 0\n";
        // CYCLOMATIC (McCabe): base 1 + one per `case_clause` (2) == 3.
        assert_eq!(cyclomatic_of(&grammar, src), 3);
        // COGNITIVE (SonarSource): the `match` structure is charged once == 1.
        assert_eq!(cognitive_of(&grammar, src), 1);
    }

    #[test]
    fn python_if_elif_else_and_try_except_follow_rules() {
        let grammar = PythonGrammar::new();
        // if +1, elif +1, else +1 == 3 (elif/else take no nesting penalty).
        let branches = "def f(x):\n    if x:\n        pass\n    elif x:\n        pass\n    else:\n        pass\n";
        assert_eq!(cognitive_of(&grammar, branches), 3);
        // try does not nest; except +1 (nesting 0) then if +1+1 == 3.
        let trycatch =
            "def f(x):\n    try:\n        pass\n    except E:\n        if x:\n            pass\n";
        assert_eq!(cognitive_of(&grammar, trycatch), 3);
    }

    // --- Canonical SonarSource example: sumOfPrimes == 7 -----------------

    #[test]
    fn ts_cognitive_sum_of_primes_equals_7() {
        let grammar = TypeScriptGrammar::new();
        // From the SonarSource Cognitive Complexity paper (expected 7):
        //   for (outer)        +1  (nesting 0)
        //     for (inner)      +2  (nesting 1)
        //       if             +3  (nesting 2)
        //         continue OUT +1  (labeled break/continue, no penalty)
        let src = "function sumOfPrimes(max) {\n\
            \x20 let total = 0;\n\
            \x20 OUT: for (let i = 1; i <= max; ++i) {\n\
            \x20   for (let j = 2; j < i; ++j) {\n\
            \x20     if (i % j === 0) {\n\
            \x20       continue OUT;\n\
            \x20     }\n\
            \x20   }\n\
            \x20   total += i;\n\
            \x20 }\n\
            \x20 return total;\n\
            }\n";
        assert_eq!(cognitive_of(&grammar, src), 7);
    }

    // --- Generator functions are captured and measured -------------------

    #[test]
    fn ts_generator_functions_are_measured() {
        use crate::adapter::queries::extract_functions;
        let grammar = TypeScriptGrammar::new();
        // Both a generator declaration and a generator expression must now be
        // extracted (TS_FUNCTION_QUERY previously captured neither) and measured
        // at nesting 0: `if (a) { if (b) {} }` == 3.
        for src in [
            "function* g() {\n  if (a) {\n    if (b) {\n    }\n  }\n}\n",
            "const g = function* () {\n  if (a) {\n    if (b) {\n    }\n  }\n};\n",
        ] {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&grammar.ts_language()).unwrap();
            let tree = parser.parse(src, None).unwrap();
            let funcs =
                extract_functions(&tree, src, std::path::Path::new("g.ts"), &grammar).unwrap();
            assert_eq!(funcs.len(), 1, "generator not extracted for: {src}");
            assert_eq!(cognitive_of(&grammar, src), 3);
        }
    }
}
