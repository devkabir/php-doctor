use crate::engine::{self, Finding, NodeContext, ScanContext};
use crate::rules::Rule;

pub struct NestedLoop;

impl Rule for NestedLoop {
    fn check(&self, scan: &ScanContext<'_, '_>, node: &NodeContext<'_>) -> Option<Finding> {
        if node.loop_depth == 0 || !engine::is_loop(node.node) {
            return None;
        }

        Some(Finding {
            code: "quadratic-nested-loop",
            message: "Nested loop may run in O(n^2) time.",
            label: "This loop is inside another loop",
            notes: &[
                "Nested iteration multiplies the amount of work done by the outer collection size.",
                "Help: Index data by key, precompute a lookup table, or collapse the loops when possible.",
            ],
            path: scan.path.to_path_buf(),
            span: node.node.byte_range(),
        })
    }
}
