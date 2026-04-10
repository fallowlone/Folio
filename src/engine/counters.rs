use std::collections::HashMap;
use super::arena::DocumentArena;
use super::styles::BoxKind;

/// Minimal heading counters collection (phase-3 foundation).
/// Returns map: block id -> numbering string (e.g. "1", "2", "3").
pub fn collect_heading_counters(styled: &DocumentArena) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut h1 = 0usize;
    for &root in &styled.roots {
        collect_from_node(styled, root, &mut h1, &mut result);
    }
    result
}

fn collect_from_node(
    styled: &DocumentArena,
    id: super::arena::NodeId,
    h1: &mut usize,
    out: &mut HashMap<String, String>,
) {
    let node = styled.get(id);
    if let BoxKind::Heading(level) = node.kind {
        if level == 1 {
            *h1 += 1;
            out.insert(node.id.clone(), format!("{}", *h1));
        }
    }

    if let super::styles::BoxContent::Children(children) = &node.content {
        for &child in children {
            collect_from_node(styled, child, h1, out);
        }
    }
}
