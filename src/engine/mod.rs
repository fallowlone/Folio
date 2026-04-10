pub mod arena;
pub mod styles;
pub mod resolver;
pub mod layout;
pub mod text;
pub mod paginate;
pub mod backend;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};
use crate::parser::ast::{Content, Document, NodeId, Value};

static RENDER_CACHE: OnceLock<Mutex<HashMap<u64, Vec<u8>>>> = OnceLock::new();
const RENDER_CACHE_LIMIT: usize = 32;

/// Полный pipeline: Document → PDF bytes
///
/// 1. Resolver:  AST → StyledTree (Arena)
/// 2. Layout:    StyledTree → LayoutTree (taffy)
/// 3. Paginate:  LayoutTree → PageTree (A4 pages)
/// 4. Backend:   PageTree → PDF bytes (pdf-writer)
pub fn render_pdf(doc: &Document) -> Vec<u8> {
    let key = document_fingerprint(doc);
    if let Some(cached) = render_cache().lock().ok().and_then(|m| m.get(&key).cloned()) {
        return cached;
    }

    let styled = resolver::build_styled_tree(doc);
    let layout = layout::compute_layout(&styled);
    let pages  = paginate::paginate(&layout, &styled);
    let pdf = backend::pdf::render(&pages);
    cache_render(key, &pdf);
    pdf
}

fn render_cache() -> &'static Mutex<HashMap<u64, Vec<u8>>> {
    RENDER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_render(key: u64, value: &[u8]) {
    if let Ok(mut map) = render_cache().lock() {
        if map.len() >= RENDER_CACHE_LIMIT {
            map.clear();
        }
        map.insert(key, value.to_vec());
    }
}

fn document_fingerprint(doc: &Document) -> u64 {
    let mut hasher = DefaultHasher::new();

    let mut vars: Vec<_> = doc.vars.iter().collect();
    vars.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
    for (k, v) in vars {
        k.hash(&mut hasher);
        hash_value(v, &mut hasher);
    }

    for &root in doc.root_ids() {
        hash_block(doc, root, &mut hasher);
    }
    hasher.finish()
}

fn hash_block(doc: &Document, id: NodeId, hasher: &mut DefaultHasher) {
    let block = doc.block(id);
    block.kind.hash(hasher);
    block.id.hash(hasher);

    let mut attrs: Vec<_> = block.attrs.iter().collect();
    attrs.sort_by(|(ka, _), (kb, _)| ka.cmp(kb));
    for (k, v) in attrs {
        k.hash(hasher);
        hash_value(v, hasher);
    }

    match &block.content {
        Content::Text(t) => {
            1u8.hash(hasher);
            t.hash(hasher);
        }
        Content::Children(children) => {
            2u8.hash(hasher);
            for &child in children {
                hash_block(doc, child, hasher);
            }
        }
        Content::Empty => {
            3u8.hash(hasher);
        }
    }
}

fn hash_value(v: &Value, hasher: &mut DefaultHasher) {
    match v {
        Value::Str(s) => {
            1u8.hash(hasher);
            s.hash(hasher);
        }
        Value::Number(n) => {
            2u8.hash(hasher);
            n.to_bits().hash(hasher);
        }
        Value::Unit(n, u) => {
            3u8.hash(hasher);
            n.to_bits().hash(hasher);
            u.hash(hasher);
        }
        Value::Var(s) => {
            4u8.hash(hasher);
            s.hash(hasher);
        }
        Value::Color(s) => {
            5u8.hash(hasher);
            s.hash(hasher);
        }
    }
}
