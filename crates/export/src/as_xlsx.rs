//! Export a PDF's pages to an OOXML Excel workbook (`.xlsx`) using a hybrid Lattice
//! (vector ruling-line grid + merged cells + multi-line cell wrapping) and Stream
//! (2D spatial row/column clustering) pipeline.

use std::fmt::Write;

use super::as_html::esc;
use super::layout::{Block, Page};
use super::zip::Zip;

/// Extracted axis-aligned vector ruling lines for one PDF page in visible page coordinates (y-up).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RulingLines {
    /// Horizontal segments: `[y, x0, x1]`.
    pub h: Vec<[f64; 3]>,
    /// Vertical segments: `[x, y0, y1]`.
    pub v: Vec<[f64; 3]>,
}

/// Convert a 0-based column index into an Excel column reference (`A`, `B`, ..., `Z`, `AA`, ...).
fn col_ref(mut col: usize) -> String {
    let mut chars = Vec::new();
    loop {
        chars.push((b'A' + (col % 26) as u8) as char);
        if col < 26 {
            break;
        }
        col = col / 26 - 1;
    }
    chars.into_iter().rev().collect()
}

#[derive(Clone, Debug)]
struct CellSpan {
    text: String,
    x0: f64,
    x1: f64,
    bold: bool,
    size: f64,
}

#[derive(Clone, Debug)]
struct SheetCell {
    col: usize,
    span: usize,
    text: String,
    style: usize,
}

struct SheetGrid {
    rows: Vec<Vec<SheetCell>>,
    col_widths: Vec<f64>,
    merges: Vec<String>,
}

// ---------------------------------------------------------------------------
// Lattice Mode: Vector ruling-line grid + merged cells + multi-line wrapping
// ---------------------------------------------------------------------------

const COORD_SNAP_TOL: f64 = 2.5;
const GAP_BRIDGE_TOL: f64 = 3.0;
const INTERSECT_TOL: f64 = 3.0;

#[derive(Clone, Debug)]
struct Segment {
    pos: f64, // y for horizontal, x for vertical
    start: f64,
    end: f64,
}

fn cluster_segments(raw: &[[f64; 3]]) -> Vec<Segment> {
    let mut segs: Vec<Segment> = raw
        .iter()
        .filter_map(|&[p, a, b]| {
            let s = a.min(b);
            let e = a.max(b);
            if e - s >= 4.0 { Some(Segment { pos: p, start: s, end: e }) } else { None }
        })
        .collect();
    segs.sort_by(|a, b| a.pos.total_cmp(&b.pos).then_with(|| a.start.total_cmp(&b.start)));

    let mut merged: Vec<Segment> = Vec::new();
    for seg in segs {
        if let Some(last) = merged.last_mut() {
            if (seg.pos - last.pos).abs() <= COORD_SNAP_TOL && seg.start <= last.end + GAP_BRIDGE_TOL {
                last.end = last.end.max(seg.end);
                last.pos = 0.5 * (last.pos + seg.pos);
                continue;
            }
        }
        merged.push(seg);
    }
    merged
}

fn segments_intersect(h: &Segment, v: &Segment) -> bool {
    v.pos >= h.start - INTERSECT_TOL
        && v.pos <= h.end + INTERSECT_TOL
        && h.pos >= v.start - INTERSECT_TOL
        && h.pos <= v.end + INTERSECT_TOL
}

fn cluster_coords(mut coords: Vec<f64>, descending: bool) -> Vec<f64> {
    if descending {
        coords.sort_by(|a, b| b.total_cmp(a));
    } else {
        coords.sort_by(f64::total_cmp);
    }
    let mut out: Vec<f64> = Vec::new();
    for c in coords {
        if out.last().is_some_and(|&prev| (c - prev).abs() <= COORD_SNAP_TOL) {
            continue;
        }
        out.push(c);
    }
    out
}

fn has_h_line(h_segs: &[Segment], y: f64, x0: f64, x1: f64) -> bool {
    let target_len = (x1 - x0).abs();
    if target_len <= 1.0 {
        return false;
    }
    let mut covered = 0.0;
    for s in h_segs {
        if (s.pos - y).abs() <= COORD_SNAP_TOL {
            let ov = (s.end.min(x1) - s.start.max(x0)).max(0.0);
            covered += ov;
        }
    }
    covered >= target_len * 0.55
}

fn has_v_line(v_segs: &[Segment], x: f64, y0: f64, y1: f64) -> bool {
    let lo = y0.min(y1);
    let hi = y0.max(y1);
    let target_len = hi - lo;
    if target_len <= 1.0 {
        return false;
    }
    let mut covered = 0.0;
    for s in v_segs {
        if (s.pos - x).abs() <= COORD_SNAP_TOL {
            let ov = (s.end.min(hi) - s.start.max(lo)).max(0.0);
            covered += ov;
        }
    }
    covered >= target_len * 0.55
}

struct LatticeTable {
    bbox: [f64; 4], // [x0, y0, x1, y1]
    xs: Vec<f64>,   // ascending
    ys: Vec<f64>,   // descending (top-to-bottom)
    h_segs: Vec<Segment>,
    v_segs: Vec<Segment>,
}

fn detect_lattice_tables(rulings: &RulingLines) -> Vec<LatticeTable> {
    let h_all = cluster_segments(&rulings.h);
    let v_all = cluster_segments(&rulings.v);
    if h_all.len() < 2 || v_all.len() < 2 {
        return Vec::new();
    }

    let mut h_vis = vec![false; h_all.len()];
    let mut v_vis = vec![false; v_all.len()];
    let mut tables = Vec::new();

    for start_h in 0..h_all.len() {
        if h_vis[start_h] {
            continue;
        }
        let mut comp_h = Vec::new();
        let mut comp_v = Vec::new();
        let mut queue_h = vec![start_h];
        let mut queue_v = Vec::new();
        h_vis[start_h] = true;

        while !queue_h.is_empty() || !queue_v.is_empty() {
            while let Some(hi) = queue_h.pop() {
                comp_h.push(h_all[hi].clone());
                for (vi, v) in v_all.iter().enumerate() {
                    if !v_vis[vi] && segments_intersect(&h_all[hi], v) {
                        v_vis[vi] = true;
                        queue_v.push(vi);
                    }
                }
            }
            while let Some(vi) = queue_v.pop() {
                comp_v.push(v_all[vi].clone());
                for (hi, h) in h_all.iter().enumerate() {
                    if !h_vis[hi] && segments_intersect(h, &v_all[vi]) {
                        h_vis[hi] = true;
                        queue_h.push(hi);
                    }
                }
            }
        }

        let xs = cluster_coords(comp_v.iter().map(|s| s.pos).collect(), false);
        let ys = cluster_coords(comp_h.iter().map(|s| s.pos).collect(), true);
        if xs.len() < 2 || ys.len() < 2 {
            continue;
        }
        let (Some(&tx0), Some(&tx1), Some(&ty1), Some(&ty0)) = (xs.first(), xs.last(), ys.first(), ys.last()) else {
            continue;
        };
        if tx1 - tx0 < 12.0 || ty1 - ty0 < 8.0 {
            continue;
        }
        // Require a real multi-cell grid (>= 2 rows or >= 2 columns) so 1x1 page frames
        // or decorative callout boxes do not swallow an entire page into cell A1.
        let n_rows = ys.len() - 1;
        let n_cols = xs.len() - 1;
        if n_rows < 2 && n_cols < 2 {
            continue;
        }

        tables.push(LatticeTable {
            bbox: [tx0, ty0, tx1, ty1],
            xs,
            ys,
            h_segs: comp_h,
            v_segs: comp_v,
        });
    }

    // Sort tables top-to-bottom.
    tables.sort_by(|a, b| b.bbox[3].total_cmp(&a.bbox[3]));
    tables
}

/// Join multiple `Block`s inside a single table cell into multi-line text (`\n` between visual lines).
fn join_cell_blocks(mut cell_blocks: Vec<&Block>) -> (String, bool) {
    if cell_blocks.is_empty() {
        return (String::new(), false);
    }
    cell_blocks.sort_by(|a, b| {
        let cy_a = (a.rect[1] + a.rect[3]) * 0.5;
        let cy_b = (b.rect[1] + b.rect[3]) * 0.5;
        cy_b.total_cmp(&cy_a).then_with(|| a.rect[0].total_cmp(&b.rect[0]))
    });

    let mut lines: Vec<Vec<&Block>> = Vec::new();
    for b in &cell_blocks {
        let cy = (b.rect[1] + b.rect[3]) * 0.5;
        let h = (b.rect[3] - b.rect[1]).abs().max(b.size).max(6.0);
        let same_line = lines.last().and_then(|row| row.first()).is_some_and(|rep| {
            let r_cy = (rep.rect[1] + rep.rect[3]) * 0.5;
            let r_h = (rep.rect[3] - rep.rect[1]).abs().max(rep.size).max(6.0);
            (cy - r_cy).abs() <= h.min(r_h) * 0.45
        });
        if same_line {
            if let Some(last) = lines.last_mut() {
                last.push(b);
            }
        } else {
            lines.push(vec![b]);
        }
    }

    let mut out = String::new();
    let mut all_bold = true;
    for (li, mut line) in lines.into_iter().enumerate() {
        line.sort_by(|a, b| a.rect[0].total_cmp(&b.rect[0]));
        if li > 0 {
            out.push('\n');
        }
        let mut first_in_line = true;
        for b in line {
            let t = b.text.trim();
            if t.is_empty() {
                continue;
            }
            all_bold &= b.bold;
            if !first_in_line && !out.ends_with(' ') && !out.ends_with('\n') {
                out.push(' ');
            }
            out.push_str(t);
            first_in_line = false;
        }
    }
    (out, all_bold)
}

struct BuiltRegion {
    y_top: f64,
    rows: Vec<Vec<SheetCell>>,
    col_widths: Vec<f64>,
    /// Relative merges: `(r0, c0, r1, c1)` 0-based within this region's `rows`.
    merges: Vec<(usize, usize, usize, usize)>,
}

fn build_lattice_region(table: &LatticeTable, blocks: &[&Block], used: &mut [bool]) -> Option<BuiltRegion> {
    let n_rows = table.ys.len().saturating_sub(1);
    let n_cols = table.xs.len().saturating_sub(1);
    if n_rows == 0 || n_cols == 0 {
        return None;
    }

    // Check if any unused block falls inside the table bbox.
    let [tx0, ty0, tx1, ty1] = table.bbox;
    let mut table_block_indices = Vec::new();
    for (idx, b) in blocks.iter().enumerate() {
        if used[idx] {
            continue;
        }
        let cx = (b.rect[0] + b.rect[2]) * 0.5;
        let cy = (b.rect[1] + b.rect[3]) * 0.5;
        if cx >= tx0 - 2.5 && cx <= tx1 + 2.5 && cy >= ty0 - 2.5 && cy <= ty1 + 2.5 {
            table_block_indices.push(idx);
        }
    }
    if table_block_indices.is_empty() {
        return None;
    }

    for &idx in &table_block_indices {
        used[idx] = true;
    }

    let mut visited = vec![vec![false; n_cols]; n_rows];
    let mut rows = vec![Vec::<SheetCell>::new(); n_rows];
    let mut col_widths = vec![10.0_f64; n_cols];
    let mut merges = Vec::new();
    let mut block_assigned = vec![false; table_block_indices.len()];

    for r in 0..n_rows {
        for c in 0..n_cols {
            if visited[r][c] {
                continue;
            }
            // Expand colspan rightwards where vertical divider is absent.
            let mut colspan = 1usize;
            while c + colspan < n_cols
                && !visited[r][c + colspan]
                && !has_v_line(&table.v_segs, table.xs[c + colspan], table.ys[r + 1], table.ys[r])
            {
                colspan += 1;
            }

            // Expand rowspan downwards where horizontal divider is absent across every sub-column.
            let mut rowspan = 1usize;
            'row_expand: while r + rowspan < n_rows {
                for cc in c..(c + colspan) {
                    if visited[r + rowspan][cc]
                        || has_h_line(&table.h_segs, table.ys[r + rowspan], table.xs[cc], table.xs[cc + 1])
                    {
                        break 'row_expand;
                    }
                }
                for cc in (c + 1)..(c + colspan) {
                    if has_v_line(&table.v_segs, table.xs[cc], table.ys[r + rowspan + 1], table.ys[r + rowspan]) {
                        break 'row_expand;
                    }
                }
                rowspan += 1;
            }

            for rr in r..(r + rowspan) {
                for cc in c..(c + colspan) {
                    visited[rr][cc] = true;
                }
            }

            let cell_x0 = table.xs[c];
            let cell_x1 = table.xs[c + colspan];
            let cell_y1 = table.ys[r];
            let cell_y0 = table.ys[r + rowspan];

            let mut cell_blocks = Vec::new();
            for (bi, &blk_idx) in table_block_indices.iter().enumerate() {
                if block_assigned[bi] {
                    continue;
                }
                let b = blocks[blk_idx];
                let cx = (b.rect[0] + b.rect[2]) * 0.5;
                let cy = (b.rect[1] + b.rect[3]) * 0.5;
                if cx >= cell_x0 - 2.5 && cx <= cell_x1 + 2.5 && cy >= cell_y0 - 2.5 && cy <= cell_y1 + 2.5 {
                    block_assigned[bi] = true;
                    cell_blocks.push(b);
                }
            }

            let (text, all_bold) = join_cell_blocks(cell_blocks);
            let is_header = r == 0 || (!text.is_empty() && all_bold);
            let style = if is_header { 1 } else { 2 };

            if colspan == 1 {
                let max_line_chars = text.lines().map(|l| l.chars().count()).max().unwrap_or(0) as f64;
                col_widths[c] = col_widths[c].max((max_line_chars + 3.0).clamp(10.0, 52.0));
            }
            if colspan > 1 || rowspan > 1 {
                merges.push((r, c, r + rowspan - 1, c + colspan - 1));
            }

            rows[r].push(SheetCell {
                col: c,
                span: colspan,
                text,
                style,
            });
            // Emit empty styled cells on spanned subsequent rows so borders render across the full merge.
            for rr in (r + 1)..(r + rowspan) {
                for cc in c..(c + colspan) {
                    rows[rr].push(SheetCell {
                        col: cc,
                        span: 1,
                        text: String::new(),
                        style,
                    });
                }
            }
        }
    }

    for row in &mut rows {
        row.sort_by_key(|c| c.col);
    }

    Some(BuiltRegion {
        y_top: ty1,
        rows,
        col_widths,
        merges,
    })
}

// ---------------------------------------------------------------------------
// Stream Mode: Spatial row/column clustering for borderless content
// ---------------------------------------------------------------------------

fn build_stream_region(blocks: &[&Block]) -> Option<BuiltRegion> {
    if blocks.is_empty() {
        return None;
    }

    let mut sorted = blocks.to_vec();
    sorted.sort_by(|a, b| {
        let cy_a = (a.rect[1] + a.rect[3]) * 0.5;
        let cy_b = (b.rect[1] + b.rect[3]) * 0.5;
        cy_b.total_cmp(&cy_a).then_with(|| a.rect[0].total_cmp(&b.rect[0]))
    });

    let y_top = sorted.first().map_or(0.0, |b| b.rect[3]);

    // 1. Group blocks into horizontal visual rows.
    let mut raw_rows: Vec<Vec<&Block>> = Vec::new();
    for b in sorted {
        let y0 = b.rect[1];
        let y1 = b.rect[3];
        let cy = (y0 + y1) * 0.5;
        let h = (y1 - y0).abs().max(b.size).max(6.0);

        let same_row = raw_rows.last().is_some_and(|row| {
            let Some(rep) = row.first() else { return false };
            let r_y0 = rep.rect[1];
            let r_y1 = rep.rect[3];
            let r_cy = (r_y0 + r_y1) * 0.5;
            let r_h = (r_y1 - r_y0).abs().max(rep.size).max(6.0);
            let overlap = (y1.min(r_y1) - y0.max(r_y0)).max(0.0);
            (cy - r_cy).abs() <= h.min(r_h) * 0.5 || overlap >= h.min(r_h) * 0.4
        });

        if same_row {
            if let Some(last) = raw_rows.last_mut() {
                last.push(b);
            }
        } else {
            raw_rows.push(vec![b]);
        }
    }

    // 2. Within each row, sort left-to-right and merge blocks that belong to the same cell.
    let mut cell_rows: Vec<Vec<CellSpan>> = Vec::with_capacity(raw_rows.len());
    for mut row in raw_rows {
        row.sort_by(|a, b| a.rect[0].total_cmp(&b.rect[0]));
        let mut cells: Vec<CellSpan> = Vec::new();
        for b in row {
            let trimmed = b.text.trim();
            if trimmed.is_empty() {
                continue;
            }
            let gap_limit = (b.size * 0.9).max(9.0);
            let merged = cells.last_mut().is_some_and(|prev| {
                let gap = b.rect[0] - prev.x1;
                if gap <= gap_limit {
                    if !prev.text.ends_with(' ') && !trimmed.starts_with(' ') {
                        prev.text.push(' ');
                    }
                    prev.text.push_str(trimmed);
                    prev.x1 = prev.x1.max(b.rect[2]);
                    prev.bold &= b.bold;
                    true
                } else {
                    false
                }
            });
            if !merged {
                cells.push(CellSpan {
                    text: trimmed.to_owned(),
                    x0: b.rect[0],
                    x1: b.rect[2],
                    bold: b.bold,
                    size: b.size,
                });
            }
        }
        if !cells.is_empty() {
            cell_rows.push(cells);
        }
    }

    // 3. Cluster X0 coordinates from multi-cell rows (or all rows if single-column).
    let mut x_anchors: Vec<f64> = cell_rows
        .iter()
        .filter(|r| r.len() >= 2)
        .flat_map(|r| r.iter().map(|c| c.x0))
        .collect();
    if x_anchors.is_empty() {
        x_anchors = cell_rows.iter().flat_map(|r| r.iter().map(|c| c.x0)).collect();
    }
    x_anchors.sort_by(f64::total_cmp);

    let mut col_starts: Vec<f64> = Vec::new();
    for x in x_anchors {
        if col_starts.last().is_some_and(|&prev| (x - prev).abs() <= 14.0) {
            continue;
        }
        col_starts.push(x);
    }
    if col_starts.is_empty() {
        col_starts.push(0.0);
    }

    let body_size = cell_rows
        .iter()
        .flat_map(|r| r.iter().map(|c| c.size))
        .fold(11.0_f64, |acc, s| acc.min(s));

    // 4. Map each cell to its column index, span, and style.
    let num_cols = col_starts.len();
    let mut col_widths = vec![10.0_f64; num_cols];
    let mut rows: Vec<Vec<SheetCell>> = Vec::with_capacity(cell_rows.len());
    let mut merges = Vec::new();

    for (ri, r) in cell_rows.iter().enumerate() {
        let is_table_row = r.len() >= 2;
        let prev_is_table = ri > 0 && cell_rows[ri - 1].len() >= 2;
        let next_is_table = ri + 1 < cell_rows.len() && cell_rows[ri + 1].len() >= 2;
        let in_table = is_table_row || (prev_is_table && next_is_table);
        let is_header_row = is_table_row && !prev_is_table && (next_is_table || r.iter().all(|c| c.bold));

        let mut placed: Vec<SheetCell> = Vec::with_capacity(r.len());
        let mut min_next_col = 0usize;
        for (ci, c) in r.iter().enumerate() {
            if min_next_col >= num_cols {
                if let Some(last) = placed.last_mut() {
                    if !last.text.is_empty() && !c.text.is_empty() {
                        last.text.push(' ');
                    }
                    last.text.push_str(&c.text);
                }
                continue;
            }
            let mut best_col = min_next_col;
            let mut best_dist = f64::INFINITY;
            for (k, &cx) in col_starts.iter().enumerate().skip(min_next_col) {
                let d = (c.x0 - cx).abs();
                if d < best_dist {
                    best_dist = d;
                    best_col = k;
                }
            }
            if best_col >= num_cols {
                best_col = num_cols - 1;
            }

            let next_cell_col = if ci + 1 < r.len() {
                let nx = r[ci + 1].x0;
                col_starts
                    .iter()
                    .enumerate()
                    .skip(best_col + 1)
                    .min_by(|(_, a), (_, b)| (nx - **a).abs().total_cmp(&(nx - **b).abs()))
                    .map_or(num_cols, |(k, _)| k)
            } else {
                num_cols
            };
            let mut span = 1usize;
            if num_cols > 1 && next_cell_col > best_col + 1 {
                for k in (best_col + 1)..next_cell_col {
                    if c.x1 > col_starts[k] + 8.0 {
                        span += 1;
                    } else {
                        break;
                    }
                }
            }
            if span > 1 {
                merges.push((ri, best_col, ri, best_col + span - 1));
            }

            let style = if is_header_row {
                1
            } else if in_table {
                2
            } else if c.bold || c.size > body_size * 1.2 {
                3
            } else {
                0
            };

            if span == 1 {
                let max_line_chars = c.text.lines().map(|l| l.chars().count()).max().unwrap_or(0) as f64;
                col_widths[best_col] = col_widths[best_col].max((max_line_chars + 3.0).clamp(10.0, 52.0));
            }
            placed.push(SheetCell {
                col: best_col,
                span,
                text: c.text.clone(),
                style,
            });
            min_next_col = (best_col + span).min(num_cols);
        }
        rows.push(placed);
    }

    Some(BuiltRegion {
        y_top,
        rows,
        col_widths,
        merges,
    })
}

/// Hybrid page grid builder: Lattice Mode for bordered vector tables, Stream Mode for remaining blocks.
fn build_page_grid_with_rulings(page: &Page, rulings: &RulingLines) -> SheetGrid {
    let blocks: Vec<&Block> = page.blocks.iter().filter(|b| !b.text.trim().is_empty()).collect();
    if blocks.is_empty() {
        return SheetGrid { rows: Vec::new(), col_widths: Vec::new(), merges: Vec::new() };
    }

    let tables = detect_lattice_tables(rulings);
    let mut used = vec![false; blocks.len()];
    let mut regions: Vec<BuiltRegion> = Vec::new();
    let mut kept_intervals: Vec<(f64, f64)> = Vec::new();

    for table in &tables {
        if let Some(region) = build_lattice_region(table, &blocks, &mut used) {
            kept_intervals.push((table.bbox[1], table.bbox[3]));
            regions.push(region);
        }
    }

    // Group remaining non-table blocks into vertical stream bands separated by lattice tables.
    let mut remaining: Vec<&Block> = blocks
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if !used[i] { Some(b) } else { None })
        .collect();
    remaining.sort_by(|a, b| b.rect[3].total_cmp(&a.rect[3]));

    if regions.is_empty() {
        if let Some(stream) = build_stream_region(&remaining) {
            regions.push(stream);
        }
    } else if !remaining.is_empty() {
        // Partition remaining blocks relative to kept table Y-intervals.
        kept_intervals.sort_by(|a, b| b.1.total_cmp(&a.1));

        let mut band_blocks: Vec<Vec<&Block>> = vec![Vec::new(); kept_intervals.len() + 1];
        for b in remaining {
            let cy = (b.rect[1] + b.rect[3]) * 0.5;
            let mut placed = false;
            for (ti, &(_, ty1)) in kept_intervals.iter().enumerate() {
                if cy >= ty1 {
                    band_blocks[ti].push(b);
                    placed = true;
                    break;
                }
            }
            if !placed {
                if let Some(last) = band_blocks.last_mut() {
                    last.push(b);
                }
            }
        }
        for band in band_blocks {
            if let Some(stream) = build_stream_region(&band) {
                regions.push(stream);
            }
        }
    }

    regions.sort_by(|a, b| b.y_top.total_cmp(&a.y_top));

    let max_cols = regions.iter().map(|r| r.col_widths.len()).max().unwrap_or(0);
    let mut col_widths = vec![10.0_f64; max_cols];
    let mut rows = Vec::new();
    let mut merges = Vec::new();

    for reg in regions {
        let row_offset = rows.len();
        for (ci, &w) in reg.col_widths.iter().enumerate() {
            if ci < col_widths.len() {
                col_widths[ci] = col_widths[ci].max(w);
            }
        }
        for (r0, c0, r1, c1) in reg.merges {
            let start = format!("{}{}", col_ref(c0), row_offset + r0 + 1);
            let end = format!("{}{}", col_ref(c1), row_offset + r1 + 1);
            merges.push(format!("{start}:{end}"));
        }
        rows.extend(reg.rows);
    }

    SheetGrid { rows, col_widths, merges }
}

fn is_excel_number(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() || t.starts_with('0') && t.len() > 1 && !t.starts_with("0.") {
        return false;
    }
    t.parse::<f64>().is_ok_and(|v| v.is_finite())
}

/// Export `pages` as a formatted `.xlsx` workbook (one worksheet per PDF page).
pub fn xlsx(pages: &[Page], title: &str) -> Vec<u8> {
    xlsx_with_rulings(pages, &[], title)
}

/// Export `pages` as a formatted `.xlsx` workbook using extracted vector `rulings`
/// for Lattice table detection and Stream clustering for borderless content.
pub fn xlsx_with_rulings(pages: &[Page], rulings: &[RulingLines], _title: &str) -> Vec<u8> {
    let empty_ruling = RulingLines::default();
    let grids: Vec<SheetGrid> = if pages.is_empty() {
        vec![SheetGrid { rows: Vec::new(), col_widths: Vec::new(), merges: Vec::new() }]
    } else {
        pages
            .iter()
            .enumerate()
            .map(|(i, p)| build_page_grid_with_rulings(p, rulings.get(i).unwrap_or(&empty_ruling)))
            .collect()
    };

    let mut z = Zip::default();

    // [Content_Types].xml
    let mut ct = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>\n\
         <Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\n\
         <Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>\n",
    );
    for i in 1..=grids.len() {
        let _ = writeln!(
            ct,
            "<Override PartName=\"/xl/worksheets/sheet{i}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>"
        );
    }
    ct.push_str("</Types>");
    z.add("[Content_Types].xml", ct.as_bytes(), true);

    // _rels/.rels
    z.add(
        "_rels/.rels",
        b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
          <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
          <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>\n\
          </Relationships>",
        true,
    );

    // xl/styles.xml:
    // 0: regular wrapped
    // 1: table header (bold + light shading + thin border + wrapped)
    // 2: table cell (thin border + wrapped)
    // 3: bold heading (12pt bold + wrapped)
    z.add(
        "xl/styles.xml",
        b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
          <styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\n\
          <fonts count=\"3\">\n\
          <font><sz val=\"11\"/><name val=\"Calibri\"/></font>\n\
          <font><b/><sz val=\"11\"/><name val=\"Calibri\"/></font>\n\
          <font><b/><sz val=\"12\"/><name val=\"Calibri\"/></font>\n\
          </fonts>\n\
          <fills count=\"3\">\n\
          <fill><patternFill patternType=\"none\"/></fill>\n\
          <fill><patternFill patternType=\"gray125\"/></fill>\n\
          <fill><patternFill patternType=\"solid\"><fgColor rgb=\"FFEEF2F6\"/></patternFill></fill>\n\
          </fills>\n\
          <borders count=\"2\">\n\
          <border><left/><right/><top/><bottom/><diagonal/></border>\n\
          <border>\n\
            <left style=\"thin\"><color rgb=\"FFD0D7DE\"/></left>\n\
            <right style=\"thin\"><color rgb=\"FFD0D7DE\"/></right>\n\
            <top style=\"thin\"><color rgb=\"FFD0D7DE\"/></top>\n\
            <bottom style=\"thin\"><color rgb=\"FFD0D7DE\"/></bottom>\n\
            <diagonal/>\n\
          </border>\n\
          </borders>\n\
          <cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>\n\
          <cellXfs count=\"4\">\n\
          <xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyAlignment=\"1\"><alignment vertical=\"top\" wrapText=\"1\"/></xf>\n\
          <xf numFmtId=\"0\" fontId=\"1\" fillId=\"2\" borderId=\"1\" xfId=\"0\" applyFont=\"1\" applyFill=\"1\" applyBorder=\"1\" applyAlignment=\"1\"><alignment vertical=\"top\" wrapText=\"1\"/></xf>\n\
          <xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"1\" xfId=\"0\" applyBorder=\"1\" applyAlignment=\"1\"><alignment vertical=\"top\" wrapText=\"1\"/></xf>\n\
          <xf numFmtId=\"0\" fontId=\"2\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\" applyAlignment=\"1\"><alignment vertical=\"top\" wrapText=\"1\"/></xf>\n\
          </cellXfs>\n\
          </styleSheet>",
        true,
    );

    // xl/workbook.xml & xl/_rels/workbook.xml.rels
    let mut wb = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\n\
         <sheets>\n",
    );
    let mut wb_rels = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n",
    );

    for (idx, grid) in grids.iter().enumerate() {
        let n = idx + 1;
        let _ = writeln!(wb, "<sheet name=\"Page {n}\" sheetId=\"{n}\" r:id=\"rId{n}\"/>");
        let _ = writeln!(
            wb_rels,
            "<Relationship Id=\"rId{n}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{n}.xml\"/>"
        );

        let mut ws = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\n",
        );
        if !grid.col_widths.is_empty() {
            ws.push_str("<cols>\n");
            for (ci, &w) in grid.col_widths.iter().enumerate() {
                let c_idx = ci + 1;
                let _ = writeln!(ws, "<col min=\"{c_idx}\" max=\"{c_idx}\" width=\"{w:.1}\" customWidth=\"1\"/>");
            }
            ws.push_str("</cols>\n");
        }
        ws.push_str("<sheetData>\n");
        for (ri, row) in grid.rows.iter().enumerate() {
            let r_num = ri + 1;
            let _ = writeln!(ws, "<row r=\"{r_num}\">");
            for cell in row {
                let c_ref = format!("{}{r_num}", col_ref(cell.col));
                if cell.text.is_empty() {
                    let _ = writeln!(ws, "<c r=\"{c_ref}\" s=\"{}\"/>", cell.style);
                } else if is_excel_number(&cell.text) {
                    let _ = writeln!(ws, "<c r=\"{c_ref}\" s=\"{}\"><v>{}</v></c>", cell.style, cell.text.trim());
                } else {
                    let _ = writeln!(
                        ws,
                        "<c r=\"{c_ref}\" t=\"inlineStr\" s=\"{}\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                        cell.style,
                        esc(&cell.text)
                    );
                }
                // Emit empty styled cells for spanned columns so borders render across the full merge.
                if cell.span > 1 && cell.style != 0 {
                    for extra in 1..cell.span {
                        let e_ref = format!("{}{r_num}", col_ref(cell.col + extra));
                        let _ = writeln!(ws, "<c r=\"{e_ref}\" s=\"{}\"/>", cell.style);
                    }
                }
            }
            ws.push_str("</row>\n");
        }
        ws.push_str("</sheetData>\n");
        if !grid.merges.is_empty() {
            let _ = writeln!(ws, "<mergeCells count=\"{}\">", grid.merges.len());
            for m in &grid.merges {
                let _ = writeln!(ws, "<mergeCell ref=\"{m}\"/>");
            }
            ws.push_str("</mergeCells>\n");
        }
        ws.push_str("</worksheet>");
        z.add(&format!("xl/worksheets/sheet{n}.xml"), ws.as_bytes(), true);
    }

    let style_rel_id = grids.len() + 1;
    let _ = writeln!(
        wb_rels,
        "<Relationship Id=\"rId{style_rel_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>"
    );
    wb.push_str("</sheets>\n</workbook>");
    wb_rels.push_str("</Relationships>");
    z.add("xl/workbook.xml", wb.as_bytes(), true);
    z.add("xl/_rels/workbook.xml.rels", wb_rels.as_bytes(), true);

    z.finish()
}
