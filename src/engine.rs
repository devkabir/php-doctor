use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use tree_sitter::{Node, Parser};

use crate::rules::{self, Rule};

#[derive(Debug)]
pub struct Finding {
    pub code: &'static str,
    pub message: &'static str,
    pub label: &'static str,
    pub notes: &'static [&'static str],
    pub path: PathBuf,
    pub span: std::ops::Range<usize>,
}

pub struct ScanContext<'source, 'path> {
    pub source: &'source str,
    pub path: &'path Path,
}

pub struct NodeContext<'tree> {
    pub loop_depth: usize,
    pub node: Node<'tree>,
}

pub struct Scanner {
    parser: Parser,
    rules: Vec<Box<dyn Rule>>,
}

impl Scanner {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_php::LANGUAGE_PHP.into())
            .context("failed to load tree-sitter PHP grammar")?;

        Ok(Self {
            parser,
            rules: rules::default_rules(),
        })
    }

    pub fn scan_path(&mut self, path: &Path, include_hidden: bool) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        for path in php_paths(path, include_hidden)? {
            findings.extend(self.scan_file(&path)?);
        }

        Ok(findings)
    }

    fn scan_file(&mut self, path: &Path) -> Result<Vec<Finding>> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let Some(tree) = self.parser.parse(&source, None) else {
            return Ok(Vec::new());
        };

        let context = ScanContext {
            source: &source,
            path,
        };
        let mut findings = Vec::new();
        self.scan_node(tree.root_node(), &context, 0, &mut findings);

        Ok(findings)
    }

    fn scan_node<'tree>(
        &self,
        node: Node<'tree>,
        context: &ScanContext<'_, '_>,
        loop_depth: usize,
        findings: &mut Vec<Finding>,
    ) {
        let node_context = NodeContext { loop_depth, node };

        for rule in &self.rules {
            if let Some(finding) = rule.check(context, &node_context) {
                findings.push(finding);
            }
        }

        let child_loop_depth = if is_loop(node) {
            loop_depth + 1
        } else {
            loop_depth
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.scan_node(child, context, child_loop_depth, findings);
        }
    }
}

pub fn php_paths(path: &Path, include_hidden: bool) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(is_php(path)
            .then(|| path.to_path_buf())
            .into_iter()
            .collect());
    }

    let mut paths = Vec::new();
    let walker = WalkBuilder::new(path)
        .hidden(!include_hidden)
        .git_ignore(true)
        .git_exclude(true)
        .parents(true)
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
            && is_php(path)
        {
            paths.push(path.to_path_buf());
        }
    }

    Ok(paths)
}

fn is_php(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("php"))
}

pub fn is_loop(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "foreach_statement" | "for_statement" | "while_statement" | "do_statement"
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn write_fixture(source: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("php-doctor-{suffix}.php"));
        fs::write(&path, source).unwrap();
        path
    }

    fn scan(source: &str) -> Vec<Finding> {
        let path = write_fixture(source);
        let mut scanner = Scanner::new().unwrap();
        let findings = scanner.scan_path(&path, false).unwrap();
        fs::remove_file(path).unwrap();
        findings
    }

    #[test]
    fn detects_nested_loop() {
        let findings = scan(
            r#"<?php
foreach ($users as $user) {
    foreach ($orders as $order) {
        echo $order->id;
    }
}
"#,
        );

        assert!(findings.iter().any(|f| f.code == "quadratic-nested-loop"));
    }

    #[test]
    fn detects_linear_search_inside_loop() {
        let findings = scan(
            r#"<?php
foreach ($ids as $id) {
    if (in_array($id, $blocked, true)) {
        continue;
    }
}
"#,
        );

        assert!(findings.iter().any(|f| f.code == "quadratic-linear-search"));
    }

    #[test]
    fn ignores_linear_search_outside_loop() {
        let findings = scan(
            r#"<?php
if (in_array($id, $blocked, true)) {
    return;
}
"#,
        );

        assert!(findings.is_empty());
    }

    #[test]
    #[ignore]
    fn dump_class_ast() {
        let source = r#"<?php
final class LedgerSchema extends Schema {
    public function a() {}
    private static function b() {}
}
"#;
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_php::LANGUAGE_PHP.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();

        println!("{}", tree.root_node().to_sexp());
    }
}
