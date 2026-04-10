use std::collections::HashMap;
use super::layout::LayoutTree;
use super::paginate::PageTree;

#[derive(Debug, Clone, Default)]
pub struct PageIntrospection {
    pub page_by_arena_id: HashMap<String, usize>,
}

/// Builds minimal introspection map for future refs/cross-links.
pub fn build_page_introspection(layout: &LayoutTree, pages: &PageTree) -> PageIntrospection {
    let mut map = HashMap::new();
    if pages.pages.is_empty() {
        return PageIntrospection { page_by_arena_id: map };
    }
    // Heuristic mapping by Y ranges: page index by absolute y.
    let page_height = pages.pages[0].height.max(1.0);
    for node in &layout.nodes {
        let page_idx = (node.y / page_height).floor().max(0.0) as usize;
        map.insert(format!("{:?}", node.arena_id), page_idx);
    }
    PageIntrospection { page_by_arena_id: map }
}
