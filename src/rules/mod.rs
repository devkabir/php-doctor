use crate::engine::{Finding, NodeContext, ScanContext};

mod linear_search_in_loop;
mod nested_loop;

pub trait Rule {
    fn check(&self, scan: &ScanContext<'_, '_>, node: &NodeContext<'_>) -> Option<Finding>;
}

pub fn default_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(nested_loop::NestedLoop),
        Box::new(linear_search_in_loop::LinearSearchInLoop),
    ]
}
