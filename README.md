# php-doctor

A fast CLI tool that scans PHP codebases for **O(n²) complexity smells** using a tree-sitter AST parser. Written in Rust.

## What it detects

| Code | Warning | Description |
|------|---------|-------------|
| `quadratic-nested-loop` | Nested loop may run in O(n²) time | A loop found inside another loop (`foreach`, `for`, `while`, `do`) |
| `quadratic-linear-search` | Repeated linear array search inside a loop | Calls to `in_array()`, `array_search()`, or `array_keys()` inside a loop |

### Example output

```
warning[quadratic-nested-loop]: Nested loop may run in O(n^2) time.
 ┌─ src/Controller.php:4:5
 │
4 │     foreach ($orders as $order) {
 │     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ This loop is inside another loop
 │
 = Nested iteration multiplies the amount of work done by the outer collection size.
 = Help: Index data by key, precompute a lookup table, or collapse the loops when possible.

warning[quadratic-linear-search]: Repeated linear array search inside a loop may run in O(n^2) time.
 ┌─ src/Controller.php:5:9
 │
5 │         if (in_array($order->id, $user->order_ids, true)) {
 │             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ This call scans an array on every loop iteration
 │
 = `in_array()`, `array_search()`, and `array_keys()` are linear in the array size.
 = Help: Build a lookup map before the loop and use `isset($map[$key])` or `array_key_exists($key, $map)`.
```

## Installation

Requires [Rust](https://rustup.rs/) (edition 2024).

```bash
git clone <repo>
cd php-scanner
cargo build --release
# binary at ./target/release/php-doctor
```

## Usage

```bash
# Scan current directory
php-doctor

# Scan a specific file or directory
php-doctor path/to/src

# Include hidden files and directories
php-doctor --hidden path/to/src
```

Respects `.gitignore` and `.git/info/exclude` — vendor directories and ignored files are skipped automatically.

## Architecture

```
src/
├── main.rs          # CLI entry point (clap), diagnostic printer
├── engine.rs        # Scanner: tree-sitter parser, AST walker, Finding type
└── rules/
    ├── mod.rs       # Rule trait + default_rules registry
    ├── nested_loop.rs          # quadratic-nested-loop
    └── linear_search_in_loop.rs # quadratic-linear-search
```

**Flow:** `Scanner::new()` loads the tree-sitter PHP grammar and all rules → `scan_path()` walks the filesystem → `scan_file()` parses each `.php` file into an AST → `scan_node()` recursively visits every node, tracking loop depth and firing each `Rule::check()` → `Finding`s are printed with source-location context.

### Adding a new rule

1. Create `src/rules/your_rule.rs` implementing the `Rule` trait:

```rust
use crate::engine::{Finding, NodeContext, ScanContext};
use crate::rules::Rule;

pub struct YourRule;

impl Rule for YourRule {
    fn check(&self, scan: &ScanContext<'_, '_>, node: &NodeContext<'_>) -> Option<Finding> {
        // inspect node.node (tree-sitter Node) and node.loop_depth
        // return Some(Finding { ... }) to report, None to pass
        todo!()
    }
}
```

2. Register it in `src/rules/mod.rs`:

```rust
mod your_rule;

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // ...existing rules...
        Box::new(your_rule::YourRule),
    ]
}
```

## Running tests

```bash
cargo test
```

Tests in `src/engine.rs` write temporary `.php` fixtures, run the scanner, and assert on finding codes.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tree-sitter` + `tree-sitter-php` | PHP AST parsing |
| `ignore` | Gitignore-aware directory walker |
| `clap` | CLI argument parsing |
| `anyhow` | Error handling |
