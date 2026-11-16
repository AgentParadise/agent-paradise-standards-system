//! Exact import resolution (issue #148).
//!
//! An import resolves to the module its path names and to nothing else. The
//! analyzer used to link an import to every module whose id shared any
//! substring with the path, so `crate::domain::contact::store` landed on
//! every `domain` and `contact` module of every crate and language.
//!
//! The rules here:
//!
//! - Resolution is by source file, per language. A Rust import never lands on
//!   a TypeScript module, and the reverse.
//! - Rust: `crate::`, `self::` and `super::` resolve inside the importing
//!   crate only; a path that starts with a workspace crate's name resolves
//!   inside that crate only. Within the crate the longest `::` prefix of the
//!   path that names a source file wins: `crate::a::b::C` is `a/b.rs` when
//!   there is no `a/b/C.rs`, and `a.rs` only when there is no `a/b.rs`.
//! - Rust files sit where the module tree puts them: every `mod name;`
//!   (with or without `#[path]`, inside inline `mod { }` blocks too) is read
//!   from the syntax tree and followed, so a binary's `mod helper;` and a
//!   `[lib] path` outside `src` resolve like the compiler loads them.
//! - Rust use trees expand to their leaves: `a::{b::{c::X}}` names `a::b::c`.
//! - TypeScript: a relative specifier (`./x`, `../y/z.js`) resolves against
//!   the importing file's directory. Bare and aliased specifiers name
//!   packages, not files, and resolve to nothing.
//! - Python imports reach the resolver flagged external (the import query
//!   does not keep the leading dots of a relative import), so they make no
//!   edge, as before.
//! - An import that names nothing analyzed resolves to nothing. There is no
//!   fallback guess.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ImportInfo;

/// `path` names `module` or something inside it, on `sep` boundaries:
/// `a::b::C` is under `a::b`, `a::bc` is not.
pub(crate) fn has_module_prefix(path: &str, module: &str, sep: &str) -> bool {
    path == module
        || path
            .strip_prefix(module)
            .is_some_and(|rest| rest.starts_with(sep))
}

/// The longest of `modules` that `path` is under on `sep` boundaries.
pub(crate) fn longest_module_prefix<'a>(
    path: &str,
    modules: impl IntoIterator<Item = &'a str>,
    sep: &str,
) -> Option<&'a str> {
    modules
        .into_iter()
        .filter(|module| !module.is_empty() && has_module_prefix(path, module, sep))
        .max_by_key(|module| module.len())
}

/// Languages that import each other's files. TypeScript and TSX share one
/// module system; every other language stands alone.
pub(crate) fn language_family(language: &str) -> &str {
    match language {
        "typescript" | "tsx" => "typescript",
        other => other,
    }
}

/// Lexical normal form of a path: `.` dropped, `..` folded into its parent.
/// Every map key in the resolver is in this form, so `./src/a.rs` and
/// `src/a.rs` are one file.
pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let last_is_normal =
                    matches!(out.components().next_back(), Some(Component::Normal(_)));
                if last_is_normal {
                    out.pop();
                } else {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// One analyzed source file.
#[derive(Debug, Clone)]
pub(crate) struct SourceFile {
    /// Grammar language id (`rust`, `typescript`, `tsx`, `python`).
    pub language: String,
    /// Module id the analyzer gives the file.
    pub module: String,
}

/// Where a Rust file sits: the compilation unit it belongs to (keyed by its
/// root directory) and its module path inside that unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RustLocation {
    unit: PathBuf,
    path: Vec<String>,
}

/// Resolves imports to the modules of analyzed files.
#[derive(Debug, Default)]
pub(crate) struct ImportResolver {
    files: HashMap<PathBuf, SourceFile>,
    rust_location: HashMap<PathBuf, RustLocation>,
    rust_file_at: HashMap<RustLocation, PathBuf>,
    /// Modules a `mod name;` declares whose file is not analyzed (the walk
    /// skips `test_*.rs`, or a config excludes it).
    unanalyzed_modules: HashSet<RustLocation>,
    /// Workspace crate name (`-` as `_`) to its library unit.
    rust_crates: HashMap<String, PathBuf>,
}

/// TypeScript source extensions a relative specifier may leave off or spell
/// as its emitted `.js` form.
const TS_EXTENSIONS: &[&str] = &["ts", "tsx", "mts", "cts"];

impl ImportResolver {
    /// Index `files` (path, language, module id) under `project_root`.
    /// Rust crate boundaries come from the nearest `Cargo.toml` with a
    /// `[package]` at or below `project_root`.
    pub(crate) fn new(
        project_root: &Path,
        files: impl IntoIterator<Item = (PathBuf, SourceFile)>,
    ) -> Self {
        let root = normalize_path(project_root);
        let mut resolver = Self::default();
        let mut manifests = ManifestCache::default();

        // Where each Rust file sits by its location on disk.
        let mut on_disk: HashMap<PathBuf, RustLocation> = HashMap::new();
        for (path, file) in files {
            let path = normalize_path(&path);
            if language_family(&file.language) == "rust" {
                let (location, crate_name) = rust_location(&path, &root, &mut manifests);
                if let Some(name) = crate_name {
                    resolver
                        .rust_crates
                        .entry(name)
                        .or_insert_with(|| location.unit.clone());
                }
                on_disk.insert(path.clone(), location);
            }
            resolver.files.insert(path, file);
        }

        // The module tree as the compiler builds it: each `mod name;` puts
        // its file at the declaring module's path plus `name`, wherever the
        // file sits on disk. `#[path = "foo_tests.rs"] mod tests;` makes
        // `foo_tests.rs` module `foo::tests`, so its `super::` is `foo`, and
        // `mod helper;` in `src/bin/tool.rs` puts `src/bin/helper.rs` in the
        // `tool` binary. A file no analyzed file declares keeps its location
        // on disk.
        let mut declaring: Vec<&PathBuf> = on_disk.keys().collect();
        declaring.sort();
        let declarations: Vec<(PathBuf, Vec<ModDeclaration>)> = declaring
            .into_iter()
            .filter_map(|file| {
                let source = fs::read_to_string(file).ok()?;
                let declared = mod_declarations(&source);
                (!declared.is_empty()).then(|| (file.clone(), declared))
            })
            .collect();
        let (mounts, unanalyzed) = mount_declarations(&declarations, &on_disk);

        for path in on_disk.keys() {
            let location = mounted_location(path, &on_disk, &mounts, 0);
            // `lib.rs` and `main.rs` of one package share the unit root;
            // keep `lib.rs`, the one other crates and `crate::` reach.
            let replace = resolver
                .rust_file_at
                .get(&location)
                .is_none_or(|existing| existing.file_stem().is_some_and(|s| s == "main"));
            if replace {
                resolver.rust_file_at.insert(location.clone(), path.clone());
            }
            resolver.rust_location.insert(path.clone(), location);
        }
        for (declaring, module_path) in unanalyzed {
            let mut location = resolver.rust_location[&declaring].clone();
            location.path.extend(module_path);
            if !resolver.rust_file_at.contains_key(&location) {
                resolver.unanalyzed_modules.insert(location);
            }
        }
        resolver
    }

    /// Module ids `import` (written in `from_file`, inside the inline module
    /// scope `scope`) names, without the importing module itself. Usually one;
    /// a Rust use list naming several modules gives one per module.
    pub(crate) fn resolve(
        &self,
        from_file: &Path,
        import: &ImportInfo,
        scope: &[String],
    ) -> Vec<String> {
        let from_file = normalize_path(from_file);
        let Some(from) = self.files.get(&from_file) else {
            return Vec::new();
        };
        let targets = match language_family(&from.language) {
            "rust" => self.resolve_rust(&from_file, import, scope),
            "typescript" if !import.is_external => self.resolve_relative_ts(&from_file, import),
            _ => Vec::new(),
        };

        let mut modules: Vec<String> = Vec::new();
        for target in targets {
            let Some(file) = self.files.get(&target) else {
                continue;
            };
            let same_language = language_family(&file.language) == language_family(&from.language);
            if same_language && file.module != from.module && !modules.contains(&file.module) {
                modules.push(file.module.clone());
            }
        }
        modules
    }

    fn resolve_rust(
        &self,
        from_file: &Path,
        import: &ImportInfo,
        scope: &[String],
    ) -> Vec<PathBuf> {
        let Some(from) = self.rust_location.get(from_file) else {
            return Vec::new();
        };
        let mut here = from.path.clone();
        here.extend(scope.iter().cloned());

        let mut targets = Vec::new();
        for full_path in rust_use_paths(import) {
            let segments: Vec<&str> = full_path.split("::").map(str::trim).collect();
            let Some((unit, path)) = self.rust_anchor(&from.unit, &here, &segments) else {
                continue;
            };
            if let Some(file) = self.longest_rust_file(unit, &path) {
                if !targets.contains(file) {
                    targets.push(file.clone());
                }
            }
        }
        targets
    }

    /// The unit and in-unit module path a Rust use path starts from, or
    /// `None` for a path outside the analyzed workspace (std, crates.io).
    fn rust_anchor<'a>(
        &'a self,
        from_unit: &'a PathBuf,
        here: &[String],
        segments: &[&str],
    ) -> Option<(&'a PathBuf, Vec<String>)> {
        let owned = |rest: &[&str]| rest.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        match segments.first().copied()? {
            "crate" => Some((from_unit, owned(&segments[1..]))),
            "self" => {
                let mut path = here.to_vec();
                path.extend(owned(&segments[1..]));
                Some((from_unit, path))
            }
            "super" => {
                let ups = segments.iter().take_while(|s| **s == "super").count();
                let kept = here.len().checked_sub(ups)?;
                let mut path = here[..kept].to_vec();
                path.extend(owned(&segments[ups..]));
                Some((from_unit, path))
            }
            // `::name::...`: a crate by absolute path.
            "" => {
                let name = segments.get(1)?;
                let unit = self.rust_crates.get(*name)?;
                Some((unit, owned(&segments[2..])))
            }
            name => {
                let unit = self.rust_crates.get(name)?;
                Some((unit, owned(&segments[1..])))
            }
        }
    }

    /// The file behind the longest prefix of `path` that names a module of
    /// `unit`; the unit's root file when no longer prefix does. `None` when
    /// the longest prefix naming a module names one that is declared or on
    /// disk but not analyzed (the walk skips `test_*.rs`): falling back to
    /// its parent would be an edge to a module the import does not name.
    fn longest_rust_file(&self, unit: &Path, path: &[String]) -> Option<&PathBuf> {
        for len in (0..=path.len()).rev() {
            let location = RustLocation {
                unit: unit.to_path_buf(),
                path: path[..len].to_vec(),
            };
            if let Some(file) = self.rust_file_at.get(&location) {
                return Some(file);
            }
            let unanalyzed = self.unanalyzed_modules.contains(&location)
                || module_file_on_disk(unit, &path[..len]);
            if len > 0 && unanalyzed {
                return None;
            }
        }
        None
    }

    fn resolve_relative_ts(&self, from_file: &Path, import: &ImportInfo) -> Vec<PathBuf> {
        let specifier = import.import_path.as_str();
        if !(specifier.starts_with("./") || specifier.starts_with("../") || specifier == ".") {
            return Vec::new();
        }
        let base = normalize_path(&from_file.parent().unwrap_or(Path::new("")).join(specifier));
        ts_candidates(&base)
            .into_iter()
            .find(|candidate| self.files.contains_key(candidate))
            .into_iter()
            .collect()
    }
}

/// A file for module `path` of `unit` exists in the default layout
/// (`a/b.rs` or `a/b/mod.rs` under the unit root).
fn module_file_on_disk(unit: &Path, path: &[String]) -> bool {
    let dir: PathBuf = path.iter().collect();
    unit.join(&dir).with_extension("rs").is_file() || unit.join(&dir).join("mod.rs").is_file()
}

/// Bound on mount chains, so a cycle of mounts terminates.
const MAX_MOUNT_DEPTH: usize = 32;

/// Bound on the rounds that settle which files are mod-rs (see
/// [`mount_declarations`]).
const MAX_MOUNT_ROUNDS: usize = 4;

/// Mounted file to the file that declares it and its module path below the
/// declaring file's module.
type Mounts = HashMap<PathBuf, (PathBuf, Vec<String>)>;

/// One file-backed `mod name;` declaration (a `mod` with no body).
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModDeclaration {
    /// Enclosing inline modules, outermost first: their name, and the
    /// directory their nested declarations look in (the name, unless a
    /// `#[path]` renames it).
    inline: Vec<(String, String)>,
    name: String,
    /// The `#[path = "..."]` value, when present.
    path: Option<String>,
}

impl ModDeclaration {
    /// Module path below the declaring file's module.
    fn module_path(&self) -> Vec<String> {
        let mut path: Vec<String> = self.inline.iter().map(|(name, _)| name.clone()).collect();
        path.push(self.name.clone());
        path
    }

    /// The analyzed file this declaration loads, per the Rust reference's
    /// module source file rules. `declaring` is mod-rs when it is a crate
    /// root or a `mod.rs`: its children sit beside it, not in a directory
    /// named after it.
    fn target(
        &self,
        declaring: &Path,
        mod_rs: bool,
        analyzed: impl Fn(&Path) -> bool,
    ) -> Option<PathBuf> {
        let dir = declaring.parent().unwrap_or(Path::new(""));
        if let (Some(path), true) = (&self.path, self.inline.is_empty()) {
            let target = normalize_path(&dir.join(path));
            return analyzed(&target).then_some(target);
        }
        let mut base = if mod_rs {
            dir.to_path_buf()
        } else {
            dir.join(declaring.file_stem()?)
        };
        base.extend(self.inline.iter().map(|(_, dir)| dir));
        let candidates = match &self.path {
            Some(path) => vec![base.join(path)],
            None => vec![
                base.join(format!("{}.rs", self.name)),
                base.join(&self.name).join("mod.rs"),
            ],
        };
        candidates
            .into_iter()
            .map(|candidate| normalize_path(&candidate))
            .find(|candidate| analyzed(candidate))
    }
}

/// Every file-backed `mod` declaration in `source`, read from the syntax
/// tree, so a declaration in a comment or string is not one. Declarations
/// inside inline `mod name { }` blocks carry that scope; `mod` items inside
/// function bodies are not followed.
fn mod_declarations(source: &str) -> Vec<ModDeclaration> {
    if !source.contains("mod") {
        return Vec::new();
    }
    let mut parser = tree_sitter::Parser::new();
    if parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .is_err()
    {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let mut declarations = Vec::new();
    collect_mod_declarations(tree.root_node(), source, &mut Vec::new(), &mut declarations);
    declarations
}

fn collect_mod_declarations(
    items: tree_sitter::Node<'_>,
    source: &str,
    inline: &mut Vec<(String, String)>,
    out: &mut Vec<ModDeclaration>,
) {
    let mut cursor = items.walk();
    for item in items.named_children(&mut cursor) {
        if item.kind() != "mod_item" {
            continue;
        }
        let Some(name) = item
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        else {
            continue;
        };
        let name = name.trim_start_matches("r#").to_string();
        let path = path_attribute(item, source);
        match item.child_by_field_name("body") {
            None => out.push(ModDeclaration {
                inline: inline.clone(),
                name,
                path,
            }),
            Some(body) => {
                let dir = path.unwrap_or_else(|| name.clone());
                inline.push((name, dir));
                collect_mod_declarations(body, source, inline, out);
                inline.pop();
            }
        }
    }
}

/// The value of a `#[path = "..."]` among the outer attributes of `item`
/// (the attribute items, and comments, right before it).
fn path_attribute(item: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    let mut sibling = item.prev_named_sibling();
    while let Some(node) = sibling {
        match node.kind() {
            "attribute_item" => {
                let attribute = node.named_child(0).filter(|a| a.kind() == "attribute");
                if let Some(attribute) = attribute {
                    let is_path = attribute
                        .named_child(0)
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        == Some("path");
                    let value = attribute
                        .child_by_field_name("value")
                        .filter(|v| v.kind() == "string_literal")
                        .and_then(|v| v.utf8_text(source.as_bytes()).ok());
                    if let (true, Some(value)) = (is_path, value) {
                        return Some(value.trim_matches('"').to_string());
                    }
                }
            }
            "line_comment" | "block_comment" => {}
            _ => break,
        }
        sibling = node.prev_named_sibling();
    }
    None
}

/// Which analyzed file each declaration mounts, and the declarations whose
/// file is not analyzed (declaring file, module path below it).
///
/// Whether a declaring file is mod-rs depends on whether something mounts
/// it (`src/bin/helper.rs` is a binary root until `tool.rs` declares
/// `mod helper;`), and its mounts depend on that; a few rounds settle it.
/// The first declaration of a file wins, in path order.
fn mount_declarations(
    declarations: &[(PathBuf, Vec<ModDeclaration>)],
    on_disk: &HashMap<PathBuf, RustLocation>,
) -> (Mounts, Vec<(PathBuf, Vec<String>)>) {
    let analyzed = |path: &Path| on_disk.contains_key(path);
    let mut mounts = Mounts::new();
    let mut unanalyzed = Vec::new();
    for _ in 0..MAX_MOUNT_ROUNDS {
        let mut next = Mounts::new();
        unanalyzed.clear();
        for (file, declared) in declarations {
            let mod_rs = file.file_name().is_some_and(|n| n == "mod.rs")
                || (on_disk[file].path.is_empty() && !mounts.contains_key(file));
            for declaration in declared {
                match declaration.target(file, mod_rs, analyzed) {
                    Some(target) if target != *file => {
                        next.entry(target)
                            .or_insert_with(|| (file.clone(), declaration.module_path()));
                    }
                    Some(_) => {}
                    None => unanalyzed.push((file.clone(), declaration.module_path())),
                }
            }
        }
        let settled =
            next.len() == mounts.len() && next.keys().all(|target| mounts.contains_key(target));
        mounts = next;
        if settled {
            break;
        }
    }
    (mounts, unanalyzed)
}

/// The location of `path`, following mounts to the declaring module. A file
/// nobody mounts keeps its location on disk.
fn mounted_location(
    path: &Path,
    on_disk: &HashMap<PathBuf, RustLocation>,
    mounts: &Mounts,
    depth: usize,
) -> RustLocation {
    match mounts.get(path) {
        Some((declaring, module_path)) if depth < MAX_MOUNT_DEPTH => {
            let mut location = mounted_location(declaring, on_disk, mounts, depth + 1);
            location.path.extend(module_path.iter().cloned());
            location
        }
        _ => on_disk[path].clone(),
    }
}

/// The full paths a Rust import names: the use path, or the use path joined
/// with each leaf of its use tree. Nested lists expand to their leaves
/// (`a::{b::{c::X}}` names `a::b::c::X`), `self` names the path it sits in,
/// `X as Y` names `X` and a glob names its path.
fn rust_use_paths(import: &ImportInfo) -> Vec<String> {
    let base = import.import_path.trim();
    if import.symbols.is_empty() {
        return vec![base.to_string()];
    }
    let mut paths = Vec::new();
    for symbol in &import.symbols {
        expand_use_tree(base, symbol, &mut paths);
    }
    paths
}

/// Push the full path of each leaf of use tree `tree`, written under `prefix`.
fn expand_use_tree(prefix: &str, tree: &str, out: &mut Vec<String>) {
    let tree = tree.trim();
    if let (Some(open), true) = (tree.find('{'), tree.ends_with('}')) {
        let path = tree[..open].trim().trim_end_matches("::").trim();
        let prefix = join_use_path(prefix, path);
        for item in split_top_level(&tree[open + 1..tree.len() - 1]) {
            expand_use_tree(&prefix, item, out);
        }
        return;
    }
    let words: Vec<&str> = tree.split_whitespace().collect();
    let leaf = match words.iter().position(|w| *w == "as") {
        Some(at) => words[..at].concat(),
        None => words.concat(),
    };
    let leaf = leaf.trim_end_matches('*').trim_end_matches("::");
    let leaf = leaf.strip_suffix("::self").unwrap_or(leaf);
    let path = if leaf == "self" {
        prefix.to_string()
    } else {
        join_use_path(prefix, leaf)
    };
    if !path.is_empty() && !out.contains(&path) {
        out.push(path);
    }
}

fn join_use_path(prefix: &str, rest: &str) -> String {
    match (prefix.is_empty(), rest.is_empty()) {
        (_, true) => prefix.to_string(),
        (true, false) => rest.to_string(),
        (false, false) => format!("{prefix}::{rest}"),
    }
}

/// The comma-separated items of a use list body, splitting only at its top
/// level.
fn split_top_level(list: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (at, c) in list.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                items.push(&list[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    items.push(&list[start..]);
    items
        .into_iter()
        .filter(|item| !item.trim().is_empty())
        .collect()
}

/// Files a TypeScript relative specifier may name, most specific first:
/// the path as written, its TS source for an emitted `.js` name, the path
/// with each TS extension, and an `index` file in the directory.
fn ts_candidates(base: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![base.to_path_buf()];
    let emitted_js = base
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e, "js" | "jsx" | "mjs" | "cjs"));
    if emitted_js {
        for ext in TS_EXTENSIONS {
            candidates.push(base.with_extension(ext));
        }
    }
    let base_str = base.to_string_lossy();
    for ext in TS_EXTENSIONS {
        candidates.push(PathBuf::from(format!("{base_str}.{ext}")));
    }
    for ext in TS_EXTENSIONS {
        candidates.push(base.join(format!("index.{ext}")));
    }
    candidates
}

/// The `[package]` facts of a `Cargo.toml` that place its library.
#[derive(Debug, Clone, Default)]
struct Package {
    /// Library name (`[lib] name`, else the package name, `-` as `_`).
    lib_name: Option<String>,
    /// `[lib] path`, relative to the package directory.
    lib_path: Option<PathBuf>,
}

/// Cached `[package]` facts of the `Cargo.toml` in a directory.
#[derive(Default)]
struct ManifestCache {
    by_dir: HashMap<PathBuf, Option<Package>>,
}

impl ManifestCache {
    /// `None`: no package manifest in `dir`.
    fn package(&mut self, dir: &Path) -> Option<Package> {
        self.by_dir
            .entry(dir.to_path_buf())
            .or_insert_with(|| read_package(dir))
            .clone()
    }
}

fn read_package(dir: &Path) -> Option<Package> {
    let manifest_path = if dir.as_os_str().is_empty() {
        PathBuf::from("Cargo.toml")
    } else {
        dir.join("Cargo.toml")
    };
    let text = fs::read_to_string(manifest_path).ok()?;
    let manifest: toml::Value = toml::from_str(&text).ok()?;
    let package = manifest.get("package")?;
    let lib = manifest.get("lib");
    let lib_field = |key: &str| lib.and_then(|lib| lib.get(key)).and_then(|v| v.as_str());
    let lib_name = lib_field("name")
        .or_else(|| package.get("name").and_then(|n| n.as_str()))
        .map(|n| n.replace('-', "_"));
    Some(Package {
        lib_name,
        lib_path: lib_field("path").map(PathBuf::from),
    })
}

/// The unit and module path of a Rust file, and the crate name when the file
/// is in a package's library unit.
fn rust_location(
    file: &Path,
    root: &Path,
    manifests: &mut ManifestCache,
) -> (RustLocation, Option<String>) {
    // The nearest package at or below the analysis root.
    let mut package: Option<(PathBuf, Package)> = None;
    for dir in file.ancestors().skip(1) {
        if let Some(found) = manifests.package(dir) {
            package = Some((dir.to_path_buf(), found));
            break;
        }
        if dir == root {
            break;
        }
    }
    let (package_dir, package) = match package {
        Some(found) => found,
        // No manifest: the parent of the nearest `src`, else the root.
        None => {
            let dir = file
                .ancestors()
                .skip(1)
                .find(|d| d.file_name().is_some_and(|n| n == "src"))
                .and_then(Path::parent)
                .unwrap_or(root)
                .to_path_buf();
            (dir, Package::default())
        }
    };

    // The library unit is `src`, or the directory of `[lib] path` when that
    // moves the library root (`code/lib.rs`). A file under both goes to the
    // deeper one.
    let src = package_dir.join("src");
    let lib_root = package
        .lib_path
        .as_ref()
        .map(|path| normalize_path(&package_dir.join(path)));
    let lib_dir = match lib_root.as_ref().and_then(|r| r.parent()) {
        Some(dir) => dir.to_path_buf(),
        None => src.clone(),
    };
    let unit = [&lib_dir, &src]
        .into_iter()
        .filter(|dir| file.starts_with(dir))
        .max_by_key(|dir| dir.components().count())
        .cloned();
    let Some(unit) = unit else {
        // Outside `src` (build.rs, examples, benches): its own root.
        return (
            RustLocation {
                unit: file.to_path_buf(),
                path: Vec::new(),
            },
            None,
        );
    };
    let relative = file.strip_prefix(&unit).unwrap_or(file);
    let mut parts: Vec<String> = relative
        .with_extension("")
        .components()
        .filter_map(|c| match c {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    // `src/bin/<name>.rs` and `src/bin/<name>/main.rs` are binary roots.
    let in_src = unit == src;
    if in_src && parts.first().is_some_and(|p| p == "bin") && parts.len() >= 2 {
        let unit = src.join("bin").join(&parts[1]);
        let mut path = parts.split_off(2);
        if path == ["main"] {
            path.clear();
        }
        if path.last().is_some_and(|p| p == "mod") {
            path.pop();
        }
        return (RustLocation { unit, path }, None);
    }

    let is_root =
        lib_root.as_deref() == Some(file) || (in_src && (parts == ["lib"] || parts == ["main"]));
    if is_root {
        parts.clear();
    }
    if parts.last().is_some_and(|p| p == "mod") {
        parts.pop();
    }
    let crate_name = package.lib_name.filter(|_| unit == lib_dir);
    (RustLocation { unit, path: parts }, crate_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ImportKind;

    #[test]
    fn module_prefix_respects_segment_boundaries() {
        assert!(has_module_prefix("a::b::C", "a::b", "::"));
        assert!(has_module_prefix("a::b", "a::b", "::"));
        assert!(!has_module_prefix("a::bc::C", "a::b", "::"));
        assert!(!has_module_prefix("a_b::C", "a", "::"));
        assert_eq!(
            longest_module_prefix("a::b::C", ["a", "a::b", "a::b::C::D", "b"], "::"),
            Some("a::b")
        );
        assert_eq!(longest_module_prefix("a_tests::X", ["a"], "::"), None);
    }

    #[test]
    fn normalize_folds_dot_segments() {
        assert_eq!(
            normalize_path(Path::new("./a/b/../c/./d.ts")),
            PathBuf::from("a/c/d.ts")
        );
        assert_eq!(normalize_path(Path::new("../x/y")), PathBuf::from("../x/y"));
    }

    #[test]
    fn mod_declarations_come_from_the_syntax_tree() {
        let source = "#[cfg(test)]\n#[path = \"a_tests.rs\"]\nmod tests;\n\
                      #[path=\"web_sign_in.rs\"] #[allow(dead_code)] pub(crate) mod sign_in;\n\
                      // #[path = \"b.rs\"] mod commented_out;\n\
                      /* mod in_block_comment; */ const S: &str = \"mod in_string;\";\n\
                      #[path = \"x.rs\"] fn not_a_mod() {}\n\
                      mod plain;\n\
                      #[path = \"renamed\"] mod outer { mod inner; #[path = \"../up.rs\"] mod up; }\n\
                      fn f() { mod in_fn; }\n";
        let declared = |name: &str, inline: &[(&str, &str)], path: Option<&str>| ModDeclaration {
            inline: inline
                .iter()
                .map(|(n, d)| (n.to_string(), d.to_string()))
                .collect(),
            name: name.to_string(),
            path: path.map(str::to_string),
        };
        assert_eq!(
            mod_declarations(source),
            [
                declared("tests", &[], Some("a_tests.rs")),
                declared("sign_in", &[], Some("web_sign_in.rs")),
                declared("plain", &[], None),
                declared("inner", &[("outer", "renamed")], None),
                declared("up", &[("outer", "renamed")], Some("../up.rs")),
            ]
        );
    }

    #[test]
    fn use_trees_expand_to_their_leaves() {
        let paths = |symbols: &[&str]| rust_use_paths(&import("crate::domain", symbols));
        assert_eq!(paths(&[]), ["crate::domain"]);
        assert_eq!(
            paths(&["contact::{store::Store}"]),
            ["crate::domain::contact::store::Store"]
        );
        assert_eq!(
            paths(&[
                "self",
                "contact::{self, store::{Store as S, Row}, *}",
                "a::b::*",
                "X as Y",
                "{c, d::E}",
            ]),
            [
                "crate::domain",
                "crate::domain::contact",
                "crate::domain::contact::store::Store",
                "crate::domain::contact::store::Row",
                "crate::domain::a::b",
                "crate::domain::X",
                "crate::domain::c",
                "crate::domain::d::E",
            ]
        );
    }

    fn import(path: &str, symbols: &[&str]) -> ImportInfo {
        ImportInfo {
            from_module: String::new(),
            to_module: String::new(),
            import_path: path.to_string(),
            is_external: !(path.starts_with('.')
                || ["crate", "self", "super"].contains(&path.split("::").next().unwrap())),
            symbols: symbols.iter().map(|s| s.to_string()).collect(),
            is_wildcard: false,
            kind: ImportKind::Single,
        }
    }

    /// A manifest-less tree: `src/` is the crate, so no file system reads
    /// matter beyond the (absent) manifests.
    fn single_crate(root: &Path) -> ImportResolver {
        let files = [
            ("src/lib.rs", "rust", "crate"),
            ("src/a.rs", "rust", "a"),
            ("src/a_b.rs", "rust", "a_b"),
            ("src/a_tests.rs", "rust", "a_tests"),
            ("src/c.rs", "rust", "c"),
            ("src/ports/mod.rs", "rust", "ports"),
            ("src/ports/x_port.rs", "rust", "ports::x_port"),
            ("config.ts", "typescript", "config"),
            ("web/form.tsx", "tsx", "web/form"),
            ("web/page.ts", "typescript", "web/page"),
            ("web/lib/index.ts", "typescript", "web/lib"),
        ];
        ImportResolver::new(
            root,
            files.iter().map(|(path, language, module)| {
                (
                    root.join(path),
                    SourceFile {
                        language: language.to_string(),
                        module: module.to_string(),
                    },
                )
            }),
        )
    }

    #[test]
    fn rust_import_resolves_to_exactly_the_named_module() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = single_crate(dir.path());
        let c = dir.path().join("src/c.rs");
        // The issue's #161 repro: `use crate::a::X` in `c` is Ce 1, not 3.
        assert_eq!(r.resolve(&c, &import("crate::a::X", &[]), &[]), ["a"]);
        assert_eq!(
            r.resolve(&c, &import("crate::ports::x_port::Port", &[]), &[]),
            ["ports::x_port"]
        );
        // Unknown leaf: longest existing prefix.
        assert_eq!(
            r.resolve(&c, &import("crate::ports::y_port::Port", &[]), &[]),
            ["ports"]
        );
        // Use lists resolve per entry; `self` names the path itself.
        assert_eq!(
            r.resolve(&c, &import("crate::ports", &["self", "x_port::P"]), &[]),
            ["ports", "ports::x_port"]
        );
        // `crate::Thing` defined in lib.rs.
        assert_eq!(r.resolve(&c, &import("crate::Thing", &[]), &[]), ["crate"]);
        // std / crates.io: nothing.
        assert!(r.resolve(&c, &import("std::fmt", &[]), &[]).is_empty());
        // Never a TypeScript `config` for a Rust `crate::config`.
        assert_eq!(
            r.resolve(&c, &import("crate::config::read", &[]), &[]),
            ["crate"]
        );
    }

    #[test]
    fn super_and_self_follow_the_inline_module_scope() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = single_crate(dir.path());
        let x_port = dir.path().join("src/ports/x_port.rs");
        assert_eq!(
            r.resolve(&x_port, &import("super::Port", &[]), &[]),
            ["ports"]
        );
        assert_eq!(
            r.resolve(&x_port, &import("super::super::a::X", &[]), &[]),
            ["a"]
        );
        // `mod tests { use super::helper; }` is the file itself: no edge.
        let tests = ["tests".to_string()];
        assert!(
            r.resolve(&x_port, &import("super::helper", &[]), &tests)
                .is_empty()
        );
        assert!(
            r.resolve(&x_port, &import("self::inner::Y", &[]), &[])
                .is_empty()
        );
        // Too many `super`s names nothing.
        assert!(
            r.resolve(&x_port, &import("super::super::super::z", &[]), &[])
                .is_empty()
        );
    }

    #[test]
    fn typescript_relative_imports_resolve_by_path() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = single_crate(dir.path());
        let page = dir.path().join("web/page.ts");
        assert_eq!(r.resolve(&page, &import("./form", &[]), &[]), ["web/form"]);
        assert_eq!(
            r.resolve(&page, &import("./form.js", &[]), &[]),
            ["web/form"]
        );
        assert_eq!(r.resolve(&page, &import("../config", &[]), &[]), ["config"]);
        assert_eq!(r.resolve(&page, &import("./lib", &[]), &[]), ["web/lib"]);
        // A file that is not analyzed, a package, a missing file: nothing.
        assert!(
            r.resolve(&page, &import("../harness/x.mjs", &[]), &[])
                .is_empty()
        );
        assert!(r.resolve(&page, &import("config", &[]), &[]).is_empty());
        assert!(r.resolve(&page, &import("./config", &[]), &[]).is_empty());
    }

    /// A resolver over real files under `root`: mounts and manifests are
    /// read from disk. Each file's module id is its path.
    fn crate_on_disk(root: &Path, files: &[(&str, &str)]) -> ImportResolver {
        for (path, source) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        ImportResolver::new(
            root,
            files
                .iter()
                .filter(|(path, _)| path.ends_with(".rs"))
                .map(|(path, _)| {
                    (
                        root.join(path),
                        SourceFile {
                            language: "rust".to_string(),
                            module: path.to_string(),
                        },
                    )
                }),
        )
    }

    #[test]
    fn nested_use_lists_resolve_to_their_leaf_modules() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = crate_on_disk(
            dir.path(),
            &[
                ("src/lib.rs", "mod app; mod domain;\n"),
                (
                    "src/app.rs",
                    "use crate::domain::{contact::{store::Store}};\n",
                ),
                ("src/domain/mod.rs", "pub mod contact;\n"),
                ("src/domain/contact/mod.rs", "pub mod store;\n"),
                ("src/domain/contact/store.rs", "pub struct Store;\n"),
            ],
        );
        let app = dir.path().join("src/app.rs");
        assert_eq!(
            r.resolve(
                &app,
                &import("crate::domain", &["contact::{store::Store}"]),
                &[]
            ),
            ["src/domain/contact/store.rs"]
        );
    }

    #[test]
    fn file_style_binaries_own_the_modules_they_declare() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = crate_on_disk(
            dir.path(),
            &[
                ("src/bin/tool.rs", "mod helper;\nuse crate::helper::run;\n"),
                ("src/bin/helper.rs", "mod nested;\npub fn run() {}\n"),
                ("src/bin/helper/nested.rs", "use super::run;\n"),
            ],
        );
        let tool = dir.path().join("src/bin/tool.rs");
        assert_eq!(
            r.resolve(&tool, &import("crate::helper::run", &[]), &[]),
            ["src/bin/helper.rs"]
        );
        // `helper.rs` is not mod-rs once mounted: its `mod nested;` is
        // `helper/nested.rs`, whose `super` is `helper`.
        let nested = dir.path().join("src/bin/helper/nested.rs");
        assert_eq!(
            r.resolve(&nested, &import("super::run", &[]), &[]),
            ["src/bin/helper.rs"]
        );
    }

    #[test]
    fn path_mounts_ignore_comments_and_honor_inline_scope() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = crate_on_disk(
            dir.path(),
            &[
                (
                    "src/lib.rs",
                    "mod a; mod b; mod c;\n\
                     mod outer { #[path = \"../mounted.rs\"] mod child; }\n",
                ),
                ("src/a.rs", "// #[path = \"b.rs\"] mod renamed;\n"),
                ("src/b.rs", "use super::c::C;\npub fn b() {}\n"),
                ("src/c.rs", "pub struct C;\n"),
                ("src/mounted.rs", "use super::super::c::C;\n"),
            ],
        );
        let b = dir.path().join("src/b.rs");
        assert_eq!(
            r.resolve(&b, &import("super::c::C", &[]), &[]),
            ["src/c.rs"]
        );
        // `src/outer/../mounted.rs` is module `outer::child`.
        let c = dir.path().join("src/c.rs");
        assert_eq!(
            r.resolve(&c, &import("crate::outer::child::X", &[]), &[]),
            ["src/mounted.rs"]
        );
        let mounted = dir.path().join("src/mounted.rs");
        assert_eq!(
            r.resolve(&mounted, &import("super::super::c::C", &[]), &[]),
            ["src/c.rs"]
        );
    }

    #[test]
    fn a_custom_library_path_is_the_crate_root() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = crate_on_disk(
            dir.path(),
            &[
                (
                    "Cargo.toml",
                    "[package]\nname = \"my-lib\"\n[lib]\npath = \"code/lib.rs\"\n",
                ),
                ("code/lib.rs", "pub mod a; pub mod b;\n"),
                ("code/a.rs", "use crate::b::B;\npub fn a() {}\n"),
                ("code/b.rs", "pub struct B;\n"),
            ],
        );
        let a = dir.path().join("code/a.rs");
        assert_eq!(
            r.resolve(&a, &import("crate::b::B", &[]), &[]),
            ["code/b.rs"]
        );
        // The crate name reaches the same unit.
        assert_eq!(
            r.resolve(&a, &import("my_lib::b::B", &[]), &[]),
            ["code/b.rs"]
        );
    }

    #[test]
    fn a_declared_module_that_is_not_analyzed_is_no_edge_to_its_parent() {
        let dir = tempfile::TempDir::new().unwrap();
        let r = crate_on_disk(
            dir.path(),
            &[
                (
                    "src/lib.rs",
                    "#[path = \"gen/out.rs\"] mod generated; mod c;\n",
                ),
                ("src/c.rs", "use crate::generated::X;\n"),
            ],
        );
        let c = dir.path().join("src/c.rs");
        assert!(
            r.resolve(&c, &import("crate::generated::X", &[]), &[])
                .is_empty()
        );
    }
}
