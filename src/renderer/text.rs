use crate::parser::ast::{Block, Content, Document};

pub fn render(doc: &Document) -> String {
    let mut out = String::new();
    for block in &doc.blocks {
        render_block(block, &mut out, 0);
    }
    out.trim().to_string()
}

fn render_block(block: &Block, out: &mut String, depth: usize) {
    match block.kind.as_str() {
        "PAGE" => {
            if depth > 0 {
                out.push_str("\n--- Page ---\n\n");
            }
            render_children(block, out, depth);
        }
        "H1" => {
            out.push_str(&format!("# {}\n\n", extract_text(block)));
        }
        "H2" => {
            out.push_str(&format!("## {}\n\n", extract_text(block)));
        }
        "H3" => {
            out.push_str(&format!("### {}\n\n", extract_text(block)));
        }
        "P" => {
            out.push_str(&format!("{}\n\n", extract_text(block)));
        }
        "GRID" => {
            render_children(block, out, depth);
        }
        _ => {
            // unknown block — render content if any
            render_children(block, out, depth);
        }
    }
}

fn render_children(block: &Block, out: &mut String, depth: usize) {
    if let Content::Blocks(blocks) = &block.content {
        for child in blocks {
            render_block(child, out, depth + 1);
        }
    }
}

fn extract_text(block: &Block) -> String {
    match &block.content {
        Content::Text(s) => s.clone(),
        Content::Empty => String::new(),
        Content::Blocks(_) => String::new(),
    }
}
