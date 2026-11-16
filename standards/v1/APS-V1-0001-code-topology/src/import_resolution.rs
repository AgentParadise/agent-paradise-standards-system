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
//! - TypeScript: a relative specifier (`./x`, `../y/z.js`) resolves against
//!   the importing file's directory. Bare and aliased specifiers name
//!   packages, not files, and resolve to nothing.
//! - Python imports reach the resolver flagged external (the import query
//!   does not keep the leading dots of a relative import), so they make no
//!   edge, as before.
//! - An import that names nothing analyzed resolves to nothing. There is no
//!   fallback guess.

use std::collections::HashMap;
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

        // A `#[path = "x.rs"] mod name;` puts `x.rs` at the declaring
        // module's path plus `name`, wherever it sits on disk: the common
        // `#[path = "foo_tests.rs"] mod tests;` makes `foo_tests.rs` module
        // `foo::tests`, so its `super::` is `foo`.
        let mut mounts: HashMap<PathBuf, (PathBuf, String)> = HashMap::new();
        for declaring in on_disk.keys() {
            let Ok(source) = fs::read_to_string(declaring) else {
                continue;
            };
            let dir = declaring.parent().unwrap_or(Path::new(""));
            for (file, name) in path_mounted_modules(&source) {
                let target = normalize_path(&dir.join(file));
                if on_disk.contains_key(&target) && target != *declaring {
                    mounts.insert(target, (declaring.clone(), name));
                }
            }
        }

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
    /// the longest prefix naming a module file names one that is on disk but
    /// not analyzed (the walk skips `test_*.rs`): falling back to its parent
    /// would be an edge to a module the import does not name.
    fn longest_rust_file(&self, unit: &Path, path: &[String]) -> Option<&PathBuf> {
        for len in (0..=path.len()).rev() {
            let location = RustLocation {
                unit: unit.to_path_buf(),
                path: path[..len].to_vec(),
            };
            if let Some(file) = self.rust_file_at.get(&location) {
                return Some(file);
            }
            if len > 0 && module_file_on_disk(unit, &path[..len]) {
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

/// Bound on `#[path]` mount chains, so a cycle of mounts terminates.
const MAX_MOUNT_DEPTH: usize = 32;

/// The location of `path`, following `#[path]` mounts to the declaring
/// module. A file nobody mounts keeps its location on disk.
fn mounted_location(
    path: &Path,
    on_disk: &HashMap<PathBuf, RustLocation>,
    mounts: &HashMap<PathBuf, (PathBuf, String)>,
    depth: usize,
) -> RustLocation {
    match mounts.get(path) {
        Some((declaring, name)) if depth < MAX_MOUNT_DEPTH => {
            let mut location = mounted_location(declaring, on_disk, mounts, depth + 1);
            location.path.push(name.clone());
            location
        }
        _ => on_disk[path].clone(),
    }
}

/// `(file, module name)` of each `#[path = "file"] mod name;` in `source`.
/// Other attributes and a visibility may sit between the two. A text scan,
/// not a parse: a mount inside an inline `mod { }` block is read as if it
/// were at file level.
fn path_mounted_modules(source: &str) -> Vec<(String, String)> {
    let mut mounts = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("#[path") {
        rest = &rest[at + "#[path".len()..];
        if let Some(mount) = parse_path_mount(rest) {
            mounts.push(mount);
        }
    }
    mounts
}

/// Parse ` = "file"] <attrs> <vis> mod name;` after a `#[path`.
fn parse_path_mount(text: &str) -> Option<(String, String)> {
    let text = text.trim_start().strip_prefix('=')?.trim_start();
    let text = text.strip_prefix('"')?;
    let end = text.find('"')?;
    let file = text[..end].to_string();
    let mut tail = text[end + 1..].trim_start().strip_prefix(']')?;
    loop {
        tail = tail.trim_start();
        if tail.starts_with("#[") {
            tail = &tail[tail.find(']')? + 1..];
        } else if let Some(after) = tail.strip_prefix("pub") {
            let after = after.trim_start();
            tail = match after.strip_prefix('(') {
                Some(scoped) => &scoped[scoped.find(')')? + 1..],
                None => after,
            };
        } else {
            break;
        }
    }
    let tail = tail.strip_prefix("mod")?;
    if !tail.starts_with(char::is_whitespace) {
        return None;
    }
    let tail = tail.trim_start();
    let name: String = tail
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let after_name = tail[name.len()..].trim_start();
    (!name.is_empty() && after_name.starts_with(';')).then_some((file, name))
}

/// The full paths a Rust import names: the use path, or the use path joined
/// with each entry of its use list (`self` naming the path itself).
fn rust_use_paths(import: &ImportInfo) -> Vec<String> {
    let base = import.import_path.trim();
    if import.symbols.is_empty() {
        return vec![base.to_string()];
    }
    import
        .symbols
        .iter()
        .map(|symbol| {
            // `X as Y` names `X`; `b::{X, Y}` names module path `b`.
            let symbol = symbol.split(" as ").next().unwrap_or(symbol).trim();
            let symbol = symbol.split("::{").next().unwrap_or(symbol).trim();
            if symbol == "self" || symbol.is_empty() {
                base.to_string()
            } else {
                format!("{base}::{symbol}")
            }
        })
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

/// Cached `[package]` facts of the `Cargo.toml` in a directory.
#[derive(Default)]
struct ManifestCache {
    by_dir: HashMap<PathBuf, Option<Option<String>>>,
}

impl ManifestCache {
    /// `None`: no package manifest in `dir`. `Some(name)`: a package whose
    /// library is `name` (`[lib] name`, else the package name, `-` as `_`),
    /// or `Some(None)` when the manifest names neither.
    fn package(&mut self, dir: &Path) -> Option<Option<String>> {
        self.by_dir
            .entry(dir.to_path_buf())
            .or_insert_with(|| read_package(dir))
            .clone()
    }
}

fn read_package(dir: &Path) -> Option<Option<String>> {
    let manifest_path = if dir.as_os_str().is_empty() {
        PathBuf::from("Cargo.toml")
    } else {
        dir.join("Cargo.toml")
    };
    let text = fs::read_to_string(manifest_path).ok()?;
    let manifest: toml::Value = toml::from_str(&text).ok()?;
    let package = manifest.get("package")?;
    let lib_name = manifest
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .and_then(|n| n.as_str());
    let name = lib_name
        .or_else(|| package.get("name").and_then(|n| n.as_str()))
        .map(|n| n.replace('-', "_"));
    Some(name)
}

/// The unit and module path of a Rust file, and the crate name when the file
/// is in a package's library unit.
fn rust_location(
    file: &Path,
    root: &Path,
    manifests: &mut ManifestCache,
) -> (RustLocation, Option<String>) {
    // The nearest package at or below the analysis root.
    let mut package: Option<(PathBuf, Option<String>)> = None;
    for dir in file.ancestors().skip(1) {
        if let Some(name) = manifests.package(dir) {
            package = Some((dir.to_path_buf(), name));
            break;
        }
        if dir == root {
            break;
        }
    }
    let (package_dir, crate_name) = match package {
        Some((dir, name)) => (dir, name),
        // No manifest: the parent of the nearest `src`, else the root.
        None => {
            let dir = file
                .ancestors()
                .skip(1)
                .find(|d| d.file_name().is_some_and(|n| n == "src"))
                .and_then(Path::parent)
                .unwrap_or(root)
                .to_path_buf();
            (dir, None)
        }
    };

    let src = package_dir.join("src");
    let Ok(relative) = file.strip_prefix(&src) else {
        // Outside `src` (build.rs, examples, benches): its own root.
        return (
            RustLocation {
                unit: file.to_path_buf(),
                path: Vec::new(),
            },
            None,
        );
    };
    let mut parts: Vec<String> = relative
        .with_extension("")
        .components()
        .filter_map(|c| match c {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    // `src/bin/<name>.rs` and `src/bin/<name>/main.rs` are binary roots.
    if parts.first().is_some_and(|p| p == "bin") && parts.len() >= 2 {
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

    if parts == ["lib"] || parts == ["main"] {
        parts.clear();
    }
    if parts.last().is_some_and(|p| p == "mod") {
        parts.pop();
    }
    (
        RustLocation {
            unit: src,
            path: parts,
        },
        crate_name,
    )
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
    fn path_mounts_are_read_through_attributes_and_visibility() {
        let source = "#[cfg(test)]\n#[path = \"a_tests.rs\"]\nmod tests;\n\
                      #[path=\"web_sign_in.rs\"] #[allow(dead_code)] pub(crate) mod sign_in;\n\
                      #[path = \"x.rs\"] fn not_a_mod() {}\n\
                      #[path = \"inline.rs\"] mod inline { }\n";
        assert_eq!(
            path_mounted_modules(source),
            [
                ("a_tests.rs".to_string(), "tests".to_string()),
                ("web_sign_in.rs".to_string(), "sign_in".to_string()),
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
}
