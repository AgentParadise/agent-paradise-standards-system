//! ST01 graph-shape patterns: `max_dependency_depth` and `max_fan_out` with
//! composition roots.
//!
//! Both read the same input as the other ST01 patterns,
//! `graphs/dependency-graph.json` under `topology_dir`, where an edge
//! `[from, to]` means "from depends on to". Both use the exception file with a
//! per-entity `value` ceiling, as threshold rules do, so a project can baseline
//! today and ratchet down. See the ST01 substandard spec §3.3 and §3.4.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{
    FitnessError, FitnessValidator, PathMatcher, Reading, RuleResult, RuleStatus, Severity,
    StaleException, StaleReason, StructuralRule, ThresholdDirection, Violation, error_codes,
    glob_matches, load_dependency_graph, tarjan_scc,
};

/// Pattern name: longest dependency path ceiling.
pub const MAX_DEPENDENCY_DEPTH: &str = "max_dependency_depth";
/// Pattern name: efferent coupling ceiling with declared composition roots.
pub const MAX_FAN_OUT: &str = "max_fan_out";

/// Field name of a dependency-depth violation.
pub const FIELD_DEPTH: &str = "dependency_depth";
/// Field name of a fan-out violation and of a root's fan-out reading.
pub const FIELD_FAN_OUT: &str = "fan_out";
/// Field name of a violation for depending on a composition root.
pub const FIELD_DEPENDS_ON_ROOT: &str = "depends_on_composition_root";

/// True when `pattern` is one of the graph-shape patterns in this module.
pub fn is_graph_shape(pattern: &str) -> bool {
    pattern == MAX_DEPENDENCY_DEPTH || pattern == MAX_FAN_OUT
}

/// Config-load checks for the graph-shape fields. A field that only a
/// graph-shape pattern reads is rejected on any other pattern, so a typo in
/// `pattern` cannot silently drop a ceiling.
pub fn validate_rule(rule: &StructuralRule) -> Result<(), String> {
    let has_roots = rule.composition_roots.is_some();
    let roots = rule.composition_roots.as_deref().unwrap_or(&[]);
    if !is_graph_shape(&rule.pattern) {
        if rule.max.is_some() || has_roots || rule.root_max.is_some() {
            return Err(format!(
                "Rule '{}': `max`, `composition_roots` and `root_max` are only valid for the \
                 `{MAX_DEPENDENCY_DEPTH}` and `{MAX_FAN_OUT}` patterns",
                rule.id
            ));
        }
        return Ok(());
    }
    if rule.max.is_none() {
        return Err(format!(
            "Rule '{}': pattern `{}` requires `max`",
            rule.id, rule.pattern
        ));
    }
    let Some(from) = &rule.from else {
        return Err(format!(
            "Rule '{}': pattern `{}` requires `from`",
            rule.id, rule.pattern
        ));
    };
    let mut globs = vec![from.path.as_str()];
    globs.extend(from.path_not.as_deref());
    if let Some(to) = &rule.to {
        globs.push(to.path.as_str());
        globs.extend(to.path_not.as_deref());
    }
    if rule.pattern == MAX_DEPENDENCY_DEPTH {
        if has_roots || rule.root_max.is_some() {
            return Err(format!(
                "Rule '{}': `composition_roots` and `root_max` are only valid for `{MAX_FAN_OUT}`",
                rule.id
            ));
        }
    } else {
        if rule.root_max.is_some() && roots.is_empty() {
            return Err(format!(
                "Rule '{}': `root_max` requires `composition_roots`",
                rule.id
            ));
        }
        globs.extend(roots.iter().map(String::as_str));
    }
    for glob in globs {
        glob::Pattern::new(glob)
            .map_err(|e| format!("Rule '{}': invalid glob '{glob}': {e}", rule.id))?;
    }
    Ok(())
}

/// Longest counted path from one node, over the condensation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthReading {
    /// Number of edges on the longest counted path.
    pub depth: usize,
    /// The path, one entry per component: a node, or a cycle as `{a, b}`.
    pub path: Vec<String>,
}

/// Dependency depth of every node in `nodes`.
///
/// An edge counts if and only if its target is in `to`. Each strongly
/// connected component counts as one node, so every member of a cycle shares
/// its component's depth. Among equally long continuations, the one whose
/// component's smallest member sorts first wins, so the reported path is
/// stable.
pub fn dependency_depths<'a>(
    nodes: &BTreeSet<&'a str>,
    edges: &BTreeSet<(&'a str, &'a str)>,
    to: &BTreeSet<&'a str>,
) -> HashMap<&'a str, DepthReading> {
    let counted: Vec<(&str, &str)> = edges
        .iter()
        .filter(|(a, b)| a != b && to.contains(b))
        .copied()
        .collect();
    let node_set: HashSet<&str> = nodes.iter().copied().collect();
    // Tarjan emits a component only after every component reachable from it,
    // so successors are always computed before the components that reach them.
    let sccs = tarjan_scc(&node_set, &counted);

    let mut members: Vec<Vec<String>> = Vec::with_capacity(sccs.len());
    let mut comp_of: HashMap<&str, usize> = HashMap::new();
    for (i, scc) in sccs.into_iter().enumerate() {
        let mut scc = scc;
        scc.sort();
        for m in &scc {
            let key = *node_set
                .get(m.as_str())
                .expect("tarjan_scc only returns nodes it was given");
            comp_of.insert(key, i);
        }
        members.push(scc);
    }

    let mut succ: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); members.len()];
    for (a, b) in &counted {
        let (ca, cb) = (comp_of[a], comp_of[b]);
        if ca != cb {
            succ[ca].insert(cb);
        }
    }

    let mut depth: Vec<Option<usize>> = vec![None; members.len()];
    let mut next: Vec<Option<usize>> = vec![None; members.len()];
    for c in 0..members.len() {
        let mut best: Option<(usize, usize)> = None;
        for &s in &succ[c] {
            let d = 1 + depth[s].expect("successor component computed first");
            let better = match best {
                None => true,
                Some((bd, bs)) => d > bd || (d == bd && members[s][0] < members[bs][0]),
            };
            if better {
                best = Some((d, s));
            }
        }
        depth[c] = Some(best.map_or(0, |(d, _)| d));
        next[c] = best.map(|(_, s)| s);
    }

    let render = |c: usize| -> String {
        if members[c].len() == 1 {
            members[c][0].clone()
        } else {
            format!("{{{}}}", members[c].join(", "))
        }
    };
    let mut out = HashMap::new();
    for &n in nodes {
        let c = comp_of[n];
        let mut path = vec![render(c)];
        let mut cur = c;
        while let Some(s) = next[cur] {
            path.push(render(s));
            cur = s;
        }
        out.insert(
            n,
            DepthReading {
                depth: depth[c].unwrap_or(0),
                path,
            },
        );
    }
    out
}

/// Efferent coupling (Ce): distinct direct targets of `node` that are in
/// `to`, self-edges excluded.
pub fn fan_out(node: &str, edges: &BTreeSet<(&str, &str)>, to: &BTreeSet<&str>) -> usize {
    edges
        .iter()
        .filter(|(a, b)| *a == node && *b != node && to.contains(b))
        .count()
}

fn selects(node: &str, m: &PathMatcher) -> bool {
    glob_matches(node, &m.path) && m.path_not.as_ref().is_none_or(|pn| !glob_matches(node, pn))
}

impl FitnessValidator {
    /// Evaluate a `max_dependency_depth` or `max_fan_out` structural rule.
    pub(crate) fn evaluate_graph_shape_rule(
        &self,
        rule: &StructuralRule,
        dimension: Option<String>,
    ) -> Result<(RuleResult, Vec<StaleException>, HashSet<String>), FitnessError> {
        let graph_path = self
            .repo_root
            .join(&self.config.config.topology_dir)
            .join("graphs/dependency-graph.json");
        let mut result = RuleResult {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            dimension,
            status: RuleStatus::Skip,
            violations: vec![],
            exceptions_used: 0,
            total_entities: None,
            readings: vec![],
        };
        if !graph_path.exists() {
            return Ok((result, vec![], HashSet::new()));
        }
        let content = std::fs::read_to_string(&graph_path)
            .map_err(|e| FitnessError::Io(graph_path.clone(), e))?;
        let graph: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| FitnessError::ParseArtifact(graph_path.clone(), e.to_string()))?;
        let (raw_nodes, raw_edges) = load_dependency_graph(&graph);

        let mut nodes: BTreeSet<&str> = raw_nodes.iter().map(String::as_str).collect();
        let edges: BTreeSet<(&str, &str)> = raw_edges
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        for (a, b) in &edges {
            nodes.insert(a);
            nodes.insert(b);
        }

        // validate_rule guarantees `from` and `max` for these patterns.
        let from_m = rule.from.as_ref().expect("validated: from");
        let max = rule.max.expect("validated: max");
        let from: BTreeSet<&str> = nodes
            .iter()
            .copied()
            .filter(|n| selects(n, from_m))
            .collect();
        let to: BTreeSet<&str> = match &rule.to {
            Some(m) => nodes.iter().copied().filter(|n| selects(n, m)).collect(),
            None => nodes.clone(),
        };
        result.total_entities = Some(from.len());

        let mut matched: HashSet<String> = HashSet::new();
        let mut violations = Vec::new();

        if rule.pattern == MAX_DEPENDENCY_DEPTH {
            let depths = dependency_depths(&nodes, &edges, &to);
            for n in &from {
                let r = &depths[n];
                if r.depth as u64 > max {
                    violations.push(self.budget_violation(
                        &rule.id,
                        n,
                        FIELD_DEPTH,
                        r.depth as f64,
                        max as f64,
                        Some(format!("longest path: {}", r.path.join(" -> "))),
                        &mut matched,
                    ));
                }
            }
        } else {
            let roots = match resolve_roots(
                rule.composition_roots.as_deref().unwrap_or(&[]),
                &nodes,
                &from,
            ) {
                Ok(r) => r,
                Err(msg) => {
                    result.status = RuleStatus::Fail;
                    result.total_entities = None;
                    result.violations = vec![Violation {
                        entity: format!("{}:{}", error_codes::INVALID_RULE, rule.id),
                        field: "composition_roots".to_string(),
                        actual: 0.0,
                        threshold: 0.0,
                        direction: ThresholdDirection::Max,
                        excepted: false,
                        message: Some(msg),
                    }];
                    // Nothing was evaluated, so no exception can be judged stale:
                    // report every entry of this rule as matched.
                    let all = self
                        .exceptions
                        .rules
                        .get(&rule.id)
                        .map(|e| e.keys().cloned().collect())
                        .unwrap_or_default();
                    return Ok((result, vec![], all));
                }
            };
            for n in &from {
                let fo = fan_out(n, &edges, &to);
                let cap = if roots.contains(n) {
                    result.readings.push(Reading {
                        entity: n.to_string(),
                        field: FIELD_FAN_OUT.to_string(),
                        value: fo as f64,
                    });
                    rule.root_max
                } else {
                    Some(max)
                };
                if let Some(cap) = cap.filter(|&c| fo as u64 > c) {
                    violations.push(self.budget_violation(
                        &rule.id,
                        n,
                        FIELD_FAN_OUT,
                        fo as f64,
                        cap as f64,
                        None,
                        &mut matched,
                    ));
                }
                for (_, r) in edges
                    .iter()
                    .filter(|(a, b)| a == n && b != n && roots.contains(b))
                {
                    let excepted = match self.exceptions.get(&rule.id, n) {
                        Some(exc) => {
                            matched.insert(n.to_string());
                            exc.targets
                                .as_ref()
                                .is_some_and(|t| t.iter().any(|t| t == r))
                        }
                        None => false,
                    };
                    violations.push(Violation {
                        entity: n.to_string(),
                        field: FIELD_DEPENDS_ON_ROOT.to_string(),
                        actual: 1.0,
                        threshold: 0.0,
                        direction: ThresholdDirection::Max,
                        excepted,
                        message: Some(format!("depends on composition root {r}")),
                    });
                }
            }
        }

        let mut stale = Vec::new();
        if let Some(entries) = self.exceptions.rules.get(&rule.id) {
            let mut keys: Vec<&String> = entries.keys().collect();
            keys.sort();
            for entity in keys {
                if !matched.contains(entity) && nodes.contains(entity.as_str()) {
                    stale.push(StaleException {
                        rule_id: rule.id.clone(),
                        entity: entity.clone(),
                        reason: StaleReason::NowPassing,
                    });
                }
            }
        }

        result.exceptions_used = violations.iter().filter(|v| v.excepted).count();
        let has_unexcepted = violations.iter().any(|v| !v.excepted);
        result.violations = violations;
        result.status = if !has_unexcepted {
            RuleStatus::Pass
        } else {
            match rule.severity.unwrap_or(self.config.config.severity_default) {
                Severity::Error => RuleStatus::Fail,
                Severity::Warning => RuleStatus::Warn,
            }
        };
        Ok((result, stale, matched))
    }

    /// A ceiling violation, excepted when the entity's exception has a
    /// `value` budget the actual fits under, or is a plain entry with neither
    /// `value` nor `targets`. A `targets`-only entry excuses root dependencies
    /// only, never a ceiling.
    #[allow(clippy::too_many_arguments)]
    fn budget_violation(
        &self,
        rule_id: &str,
        entity: &str,
        field: &str,
        actual: f64,
        threshold: f64,
        message: Option<String>,
        matched: &mut HashSet<String>,
    ) -> Violation {
        let excepted = match self.exceptions.get(rule_id, entity) {
            Some(exc) => {
                matched.insert(entity.to_string());
                match (exc.value, &exc.targets) {
                    (Some(budget), _) => actual <= budget,
                    (None, None) => true,
                    (None, Some(_)) => false,
                }
            }
            None => false,
        };
        Violation {
            entity: entity.to_string(),
            field: field.to_string(),
            actual,
            threshold,
            direction: ThresholdDirection::Max,
            excepted,
            message,
        }
    }
}

/// Resolve the `composition_roots` globs. Fails closed: a glob that matches no
/// node, or matches a node outside `from`, is a config error, so a renamed
/// root cannot silently lift the cap.
fn resolve_roots<'a>(
    globs: &[String],
    nodes: &BTreeSet<&'a str>,
    from: &BTreeSet<&'a str>,
) -> Result<BTreeSet<&'a str>, String> {
    let mut roots = BTreeSet::new();
    let mut outside: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for g in globs {
        let hits: Vec<&str> = nodes
            .iter()
            .copied()
            .filter(|n| glob_matches(n, g))
            .collect();
        if hits.is_empty() {
            return Err(format!("composition_roots glob '{g}' matches no node"));
        }
        for h in hits {
            if from.contains(h) {
                roots.insert(h);
            } else {
                outside.entry(g.as_str()).or_default().push(h);
            }
        }
    }
    if let Some((g, hs)) = outside.into_iter().next() {
        return Err(format!(
            "composition_roots glob '{g}' matches node(s) outside `from`: {}",
            hs.join(", ")
        ));
    }
    Ok(roots)
}
