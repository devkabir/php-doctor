use tree_sitter::Node;

use crate::engine::{Finding, NodeContext, ScanContext};
use crate::rules::Rule;

pub struct LinearSearchInLoop;

impl Rule for LinearSearchInLoop {
    fn check(&self, scan: &ScanContext<'_, '_>, node: &NodeContext<'_>) -> Option<Finding> {
        if node.loop_depth == 0 || !is_linear_array_search_call(node.node, scan.source) {
            return None;
        }

        Some(Finding {
            code: "quadratic-linear-search",
            message: "Repeated linear array search inside a loop may run in O(n^2) time.",
            label: "This call scans an array on every loop iteration",
            notes: &[
                "`in_array()`, `array_search()`, and `array_keys()` are linear in the array size.",
                "Help: Build a lookup map before the loop and use `isset($map[$key])` or `array_key_exists($key, $map)`.",
            ],
            path: scan.path.to_path_buf(),
            span: node.node.byte_range(),
        })
    }
}

fn is_linear_array_search_call(node: Node<'_>, source: &str) -> bool {
    if node.kind() != "function_call_expression" {
        return false;
    }

    let mut cursor = node.walk();
    node.children(&mut cursor).any(|child| {
        child.kind() == "name"
            && matches!(
                child.utf8_text(source.as_bytes()).unwrap_or_default(),
                "in_array" | "array_search" | "array_keys"
            )
    })
}
