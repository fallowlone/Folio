//! Integration checks for Document → layout → pagination.

use crate::engine::arena::{DocumentArena, NodeId};
use crate::engine::grid_tracks::GridColumnTrack;
use crate::engine::layout::{compute_layout, LayoutContent, LayoutNodeIdx, LayoutTree};
use crate::engine::paginate::{paginate, DrawCommand};
use crate::engine::resolver::build_styled_tree;
use crate::engine::{render, ExportFormat, ExportOptions};
use crate::engine::styles::{BoxContent, BoxKind, StyledBox};
use crate::lexer::Lexer;
use crate::parser::{self, Parser};

fn find_first_grid<'a>(styled: &'a DocumentArena, id: NodeId) -> Option<&'a StyledBox> {
    let node = styled.get(id);
    if matches!(node.kind, BoxKind::Grid) {
        return Some(node);
    }
    if let BoxContent::Children(children) = &node.content {
        for &cid in children {
            if let Some(g) = find_first_grid(styled, cid) {
                return Some(g);
            }
        }
    }
    None
}

fn load_fol(src: &str) -> crate::parser::ast::Document {
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let doc = parser.parse().expect("parse");
    let doc = parser::resolver::resolve(doc);
    parser::id::assign_ids(doc)
}

/// Each top-level `PAGE` must begin on a new physical page when the prior
/// page still has room (short content). Regression: merged PAGE blocks on one PDF page.
#[test]
fn five_page_blocks_yield_five_physical_pages() {
    let mut fol = String::new();
    for i in 0..5 {
        fol.push_str(&format!("PAGE(P(Page {i} short.))\n"));
    }
    let doc = load_fol(&fol);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);
    assert_eq!(
        layout.roots.len(),
        5,
        "fixture must produce one layout root per PAGE block"
    );
    let pages = paginate(&layout, &styled);
    assert_eq!(
        pages.pages.len(),
        5,
        "each PAGE block must start a new physical page"
    );
}

/// Narrow fixed column + long `P`: taffy leaf measure must wrap text so `LayoutBox` height > one line.
#[test]
fn grid_narrow_column_multiline_paragraph_measured_height() {
    let fol = r#"
PAGE(
  GRID({columns: "40pt 1fr"}
    P(Alpha Beta Gamma Delta Epsilon Zeta Eta Theta Iota Kappa Lambda Mu)
    P(X)
  )
)
"#;
    let doc = load_fol(fol);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);

    let narrow_heights: Vec<f32> = layout
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, BoxKind::Paragraph) && n.width < 55.0)
        .map(|n| n.height)
        .collect();

    assert!(
        narrow_heights.iter().any(|&h| h > 18.0),
        "expected wrapped paragraph height > one line (~13pt), got {:?}",
        narrow_heights
    );

    let _ = paginate(&layout, &styled);
}

/// GRID with `columns: "1fr 2fr"` yields cell widths in roughly a 1:2 ratio (taffy + extract_layout).
#[test]
fn grid_1fr_2fr_column_width_ratio() {
    let fol = r#"
PAGE(
  GRID({columns: "1fr 2fr"}
    P(Left)
    P(Right)
  )
)
"#;
    let doc = load_fol(fol);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);

    let mut paragraphs: Vec<(f32, f32)> = layout
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, BoxKind::Paragraph))
        .map(|n| (n.width, n.x))
        .collect();
    assert_eq!(paragraphs.len(), 2, "expected two P children");
    paragraphs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let w_left = paragraphs[0].0;
    let w_right = paragraphs[1].0;
    assert!(
        w_left > 10.0 && w_right > 10.0,
        "sane widths: left={w_left} right={w_right}"
    );
    let ratio = w_left / w_right;
    assert!(
        (ratio - 0.5).abs() < 0.08,
        "expected width ratio w_left/w_right ≈ 0.5 for 1fr:2fr, got {ratio} (left={w_left} right={w_right})"
    );

    let _ = paginate(&layout, &styled);
}

/// Unquoted `columns: 2fr` is one `2fr` track, not two `1fr` columns.
#[test]
fn grid_unquoted_2fr_single_column_track() {
    let fol = r#"
PAGE(
  GRID({columns: 2fr}
    P(Solo)
  )
)
"#;
    let doc = load_fol(fol);
    let styled = build_styled_tree(&doc);
    let root = *styled.roots.first().expect("root");
    let grid = find_first_grid(&styled, root).expect("GRID node");
    assert_eq!(
        grid.styles.grid_column_tracks,
        vec![GridColumnTrack::Fr(2.0)],
        "unquoted 2fr must be one 2fr track"
    );
    let layout = compute_layout(&styled);
    let _ = paginate(&layout, &styled);
}

/// Empty `IMAGE` / `FIGURE` leaf: engine draws a visible placeholder until raster decode exists.
#[test]
fn empty_image_yields_placeholder_rect_in_page_tree() {
    let fol = r#"PAGE(IMAGE({width: 40}))"#;
    let doc = load_fol(fol);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);
    let pages = paginate(&layout, &styled);
    let placeholders = pages
        .pages
        .iter()
        .flat_map(|p| &p.commands)
        .filter(|c| matches!(c, DrawCommand::Rect { fill: Some(_), stroke: Some(_), .. }))
        .count();
    assert!(
        placeholders >= 1,
        "expected at least one stroked filled rect as image placeholder"
    );
}

/// `{{sec}}` in heading content is expanded to the outline number (`docs/SPEC.md`).
/// Leading `{` must be escaped as `\{` so the lexer does not parse attrs.
#[test]
fn pipeline_heading_sec_placeholder_svg() {
    let fol = r#"PAGE(H1(\{{sec}} Alpha) H1(\{{sec}} Beta))"#;
    let doc = load_fol(fol);
    let bytes = render(
        &doc,
        ExportOptions {
            format: ExportFormat::Svg,
        },
    );
    let s = String::from_utf8(bytes).expect("utf8 svg");
    assert!(s.contains("Alpha"), "heading body missing: {}", &s[..s.len().min(500)]);
    assert!(s.contains("Beta"));
    assert!(s.contains(">1</text>") || s.contains(">1 <"));
    assert!(s.contains(">2</text>") || s.contains(">2 <"));
}

/// `{{page:id}}` resolves to the 1-based start page of the target block.
#[test]
fn page_map_records_explicit_heading_id() {
    let fol = r#"PAGE(H1[target](Title) P(x))"#;
    let doc = load_fol(fol);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);
    let tree = paginate(&layout, &styled);
    assert!(
        tree.block_start_page.contains_key("target"),
        "map={:?}",
        tree.block_start_page
    );
}

/// PDF export uses the same layout pipeline as native preview; header must be stable.
#[test]
fn pipeline_pdf_starts_with_magic_bytes() {
    let fol = r#"PAGE(P(Hello PDF))"#;
    let doc = load_fol(fol);
    let bytes = render(
        &doc,
        ExportOptions {
            format: ExportFormat::Pdf,
        },
    );
    assert!(
        bytes.len() >= 5 && &bytes[..5] == b"%PDF-",
        "expected PDF header %%-, got {:?}",
        bytes.get(..12.min(bytes.len()))
    );
}

/// Inline `[text](url)` must produce a PDF link annotation (clickable in viewers), not only blue text.
#[test]
fn pipeline_pdf_inline_link_emits_uri_annotation() {
    let fol = r#"PAGE(P(Visit [site](https://example.com/path) now.))"#;
    let doc = load_fol(fol);
    let bytes = render(
        &doc,
        ExportOptions {
            format: ExportFormat::Pdf,
        },
    );
    assert!(
        bytes.windows(5).any(|w| w == b"/URI "),
        "expected /URI action in PDF, len={}",
        bytes.len()
    );
    assert!(
        bytes.windows(5).any(|w| w == b"/Link"),
        "expected Link annotation subtype"
    );
}

#[test]
fn table_row_cell_boxes_share_top_y() {
    let src = r#"PAGE(
      TABLE(
        ROW(
          CELL(P(A))
          CELL(P(BB))
          CELL(P(Longer text in middle))
          CELL(P(D))
        )
      )
    )"#;
    let doc = load_fol(src);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);
    fn collect_cells(layout: &LayoutTree, idx: LayoutNodeIdx, out: &mut Vec<f32>) {
        let n = &layout.nodes[idx];
        if matches!(n.kind, BoxKind::Cell) {
            out.push(n.y);
        }
        if let LayoutContent::Children(ch) = &n.content {
            for &c in ch {
                collect_cells(layout, c, out);
            }
        }
    }
    let mut ys = Vec::new();
    collect_cells(&layout, layout.roots[0], &mut ys);
    assert!(
        ys.len() >= 4,
        "expected4 cells, ys={ys:?}"
    );
    let min = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (max - min).abs() < 0.5,
        "cells in one row should share same layout y; got {ys:?}"
    );
}

#[test]
fn table_cell_paragraph_on_second_fol_paint_y_is_page_local() {
    // First PAGE: many blocks so `compute_layout` stacks a large `offset_y` before the next root.
    // Regression: table CELL(P(...)) used raw layout `child.y` (document-absolute) as `cursor_y`,
    // placing text far below the page when a prior fol was tall.
    let mut src = String::from("PAGE(");
    for _ in 0..45 {
        src.push_str("P(Line of filler to grow first fol layout height.)\n");
    }
    src.push_str(")\nPAGE(TABLE(ROW(CELL(P(CellA)) CELL(P(CellB)))))");
    let doc = load_fol(&src);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);
    let tree = paginate(&layout, &styled);
    let last_page = tree.pages.last().expect("at least one page");
    let mut hits: Vec<f32> = Vec::new();
    for cmd in &last_page.commands {
        if let DrawCommand::Text { y, content, .. } = cmd {
            if content == "CellA" || content == "CellB" {
                hits.push(*y);
            }
        }
    }
    assert!(
        hits.len() >= 2,
        "expected both cell labels painted on last page, got {:?}",
        hits
    );
    for y in &hits {
        assert!(
            *y > 25.0 && *y < 820.0,
            "cell paragraph must paint inside A4 content band, got y={}",
            y
        );
    }
}

#[test]
fn table_row_paragraph_boxes_share_top_y_within_row() {
    let src = r#"PAGE(
      TABLE(
        ROW(
          CELL(P(A))
          CELL(P(BB))
          CELL(P(Longer text in middle))
          CELL(P(D))
        )
      )
    )"#;
    let doc = load_fol(src);
    let styled = build_styled_tree(&doc);
    let layout = compute_layout(&styled);
    fn collect_p_in_table(layout: &LayoutTree, idx: LayoutNodeIdx, out: &mut Vec<f32>) {
        let n = &layout.nodes[idx];
        if matches!(n.kind, BoxKind::Paragraph) {
            out.push(n.y);
        }
        if let LayoutContent::Children(ch) = &n.content {
            for &c in ch {
                collect_p_in_table(layout, c, out);
            }
        }
    }
    let mut ys = Vec::new();
    collect_p_in_table(&layout, layout.roots[0], &mut ys);
    assert!(ys.len() >= 4, "expected4 P nodes, ys={ys:?}");
    let min = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (max - min).abs() < 0.5,
        "paragraphs in one table row should share same y; got {ys:?}"
    );
}

#[test]
fn pipeline_page_placeholder_svg() {
    let fol = r#"PAGE(H1[target](Title) P(On page \{{page:target}}.))"#;
    let doc = load_fol(fol);
    let bytes = render(
        &doc,
        ExportOptions {
            format: ExportFormat::Svg,
        },
    );
    let s = String::from_utf8(bytes).expect("utf8 svg");
    assert!(
        s.contains(">1.<") || s.contains(">1</text>"),
        "expected substituted page digit in SVG, got: {}",
        &s[..s.len().min(1200)]
    );
}
