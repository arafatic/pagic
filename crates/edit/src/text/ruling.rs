use pagic_content::{Matrix, Op, Pieces};
use pagic_cos::{Dict, Document, Object};

use super::lines::{content_streams_within, page_dict};
use super::{DecodeBudget, EditError};

const MAX_FORM_DEPTH: usize = 8;
const MAX_FORM_VISITS: usize = 64;
const MIN_LINE_LENGTH: f64 = 4.0;
const MAX_THIN_RECT_THICKNESS: f64 = 3.5;
const AXIS_TOLERANCE: f64 = 1.0;

/// A horizontal or vertical ruling line segment in PDF page user space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RulingSegment {
    /// Bounding box `[x0, y0, x1, y1]` in page user space.
    pub rect: [f64; 4],
    /// `true` if horizontal, `false` if vertical.
    pub horizontal: bool,
}

#[derive(Default)]
struct PathState {
    segments: Vec<([f64; 2], [f64; 2])>,
    rects: Vec<[f64; 4]>,
    cur: Option<[f64; 2]>,
    start: Option<[f64; 2]>,
}

impl PathState {
    fn clear(&mut self) {
        self.segments.clear();
        self.rects.clear();
        self.cur = None;
        self.start = None;
    }

    fn close_subpath(&mut self) {
        if let (Some(p0), Some(s)) = (self.cur, self.start) {
            self.segments.push((p0, s));
            self.cur = Some(s);
        }
    }
}

struct Walker<'a> {
    doc: &'a Document,
    budget: DecodeBudget,
    visits: usize,
    out: Vec<RulingSegment>,
}

impl<'a> Walker<'a> {
    fn walk_ops(&mut self, ops: &[Op], resources: Option<&Dict>, ctm: Matrix, depth: usize) {
        let mut stack: Vec<Matrix> = Vec::new();
        let mut cur_ctm = ctm;
        let mut path = PathState::default();

        for op in ops {
            match op.op.as_slice() {
                b"q" => stack.push(cur_ctm),
                b"Q" => {
                    if let Some(prev) = stack.pop() {
                        cur_ctm = prev;
                    }
                }
                b"cm" => {
                    if let Some(m) = Matrix::from_operands(&op.operands) {
                        cur_ctm = m.then(&cur_ctm);
                    }
                }
                b"m" => {
                    if let Some([x, y]) = op.nums::<2>() {
                        path.cur = Some([x, y]);
                        path.start = Some([x, y]);
                    }
                }
                b"l" => {
                    if let Some([x, y]) = op.nums::<2>() {
                        if let Some(p0) = path.cur {
                            path.segments.push((p0, [x, y]));
                        }
                        path.cur = Some([x, y]);
                    }
                }
                b"c" => {
                    if let Some([_, _, _, _, x3, y3]) = op.nums::<6>() {
                        path.cur = Some([x3, y3]);
                    }
                }
                b"v" | b"y" => {
                    if let Some([_, _, x3, y3]) = op.nums::<4>() {
                        path.cur = Some([x3, y3]);
                    }
                }
                b"h" => path.close_subpath(),
                b"re" => {
                    if let Some([x, y, w, h]) = op.nums::<4>() {
                        path.rects.push([x, y, x + w, y + h]);
                        path.cur = Some([x, y]);
                        path.start = Some([x, y]);
                    }
                }
                b"S" | b"B" | b"B*" => {
                    self.emit_stroked(&path, &cur_ctm);
                    if matches!(op.op.as_slice(), b"B" | b"B*") {
                        self.emit_filled(&path, &cur_ctm);
                    }
                    path.clear();
                }
                b"s" | b"b" | b"b*" => {
                    path.close_subpath();
                    self.emit_stroked(&path, &cur_ctm);
                    if matches!(op.op.as_slice(), b"b" | b"b*") {
                        self.emit_filled(&path, &cur_ctm);
                    }
                    path.clear();
                }
                b"f" | b"F" | b"f*" => {
                    self.emit_filled(&path, &cur_ctm);
                    path.clear();
                }
                b"n" => path.clear(),
                b"Do" => {
                    if depth < MAX_FORM_DEPTH && self.visits < MAX_FORM_VISITS {
                        if let Some(name) = op.last_name() {
                            self.walk_form(name, resources, cur_ctm, depth + 1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn walk_form(&mut self, name: &[u8], resources: Option<&Dict>, ctm: Matrix, depth: usize) {
        let Some(res) = resources else {
            return;
        };
        let Some(xobj_obj) = res.get(b"XObject") else {
            return;
        };
        let xobj_resolved = self.doc.resolve(xobj_obj);
        let Some(xobj_dict) = xobj_resolved.as_dict() else {
            return;
        };
        let Some(form_ref) = xobj_dict.get(name) else {
            return;
        };
        let form_resolved = self.doc.resolve(form_ref);
        let Object::Stream(stream) = &*form_resolved else {
            return;
        };
        if stream.dict.get(b"Subtype").and_then(|o| o.as_name()) != Some(b"Form".as_slice()) {
            return;
        }
        let Ok(data) = self.budget.decode(stream) else {
            return;
        };
        self.visits += 1;

        let form_matrix = stream
            .dict
            .get(b"Matrix")
            .and_then(|o| match &*self.doc.resolve(o) {
                Object::Array(arr) => Matrix::from_operands(arr),
                _ => None,
            })
            .unwrap_or(Matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
        let child_ctm = form_matrix.then(&ctm);

        let form_res_owned = stream.dict.get(b"Resources").map(|o| self.doc.resolve(o));
        let child_res = form_res_owned.as_deref().and_then(|o| o.as_dict()).or(resources);

        let parsed = pagic_content::parse(&data);
        self.walk_ops(&parsed.ops, child_res, child_ctm, depth);
    }

    fn emit_stroked(&mut self, path: &PathState, ctm: &Matrix) {
        for &(a, b) in &path.segments {
            let p0 = ctm.apply(a[0], a[1]);
            let p1 = ctm.apply(b[0], b[1]);
            self.push_segment(p0, p1);
        }
        for &[x0, y0, x1, y1] in &path.rects {
            let c00 = ctm.apply(x0, y0);
            let c10 = ctm.apply(x1, y0);
            let c11 = ctm.apply(x1, y1);
            let c01 = ctm.apply(x0, y1);
            self.push_segment(c00, c10);
            self.push_segment(c10, c11);
            self.push_segment(c11, c01);
            self.push_segment(c01, c00);
        }
    }

    fn emit_filled(&mut self, path: &PathState, ctm: &Matrix) {
        for &r in &path.rects {
            let bbox = ctm.bbox(r);
            self.push_thin_rect(bbox);
        }
        if path.segments.len() >= 3 && path.segments.len() <= 5 {
            let mut min_x = f64::INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_y = f64::NEG_INFINITY;
            for &(a, b) in &path.segments {
                let p0 = ctm.apply(a[0], a[1]);
                let p1 = ctm.apply(b[0], b[1]);
                min_x = min_x.min(p0.0).min(p1.0);
                min_y = min_y.min(p0.1).min(p1.1);
                max_x = max_x.max(p0.0).max(p1.0);
                max_y = max_y.max(p0.1).max(p1.1);
            }
            if min_x.is_finite() {
                self.push_thin_rect([min_x, min_y, max_x, max_y]);
            }
        }
    }

    fn push_segment(&mut self, p0: (f64, f64), p1: (f64, f64)) {
        let dx = (p1.0 - p0.0).abs();
        let dy = (p1.1 - p0.1).abs();
        if dy <= AXIS_TOLERANCE && dx >= MIN_LINE_LENGTH {
            let y = 0.5 * (p0.1 + p1.1);
            self.out.push(RulingSegment {
                rect: [p0.0.min(p1.0), y, p0.0.max(p1.0), y],
                horizontal: true,
            });
        } else if dx <= AXIS_TOLERANCE && dy >= MIN_LINE_LENGTH {
            let x = 0.5 * (p0.0 + p1.0);
            self.out.push(RulingSegment {
                rect: [x, p0.1.min(p1.1), x, p0.1.max(p1.1)],
                horizontal: false,
            });
        }
    }

    fn push_thin_rect(&mut self, bbox: [f64; 4]) {
        let w = (bbox[2] - bbox[0]).abs();
        let h = (bbox[3] - bbox[1]).abs();
        if h <= MAX_THIN_RECT_THICKNESS && w >= MIN_LINE_LENGTH {
            let y = 0.5 * (bbox[1] + bbox[3]);
            self.out.push(RulingSegment {
                rect: [bbox[0].min(bbox[2]), y, bbox[0].max(bbox[2]), y],
                horizontal: true,
            });
        } else if w <= MAX_THIN_RECT_THICKNESS && h >= MIN_LINE_LENGTH {
            let x = 0.5 * (bbox[0] + bbox[2]);
            self.out.push(RulingSegment {
                rect: [x, bbox[1].min(bbox[3]), x, bbox[1].max(bbox[3])],
                horizontal: false,
            });
        }
    }
}

/// Extract horizontal and vertical ruling line segments from a page's vector content streams.
///
/// Captures both stroked paths (`m`/`l`/`re`/`S`) and thin filled rectangles (`re`/`f`)
/// used by PDF generators for table borders, in page user space.
pub fn page_ruling_lines(doc: &Document, page: usize) -> Result<Vec<RulingSegment>, EditError> {
    let page_obj = page_dict(doc, page)?;
    let mut budget = DecodeBudget::page();
    let streams = content_streams_within(doc, &page_obj.dict, &mut budget)?;
    let raw: Vec<&[u8]> = streams.iter().map(|(_, d)| d.as_slice()).collect();
    let pieces = Pieces::join(&raw);
    let parsed = pieces.parse();

    let res_owned = page_obj.dict.get(b"Resources").map(|o| doc.resolve(o));
    let resources = res_owned.as_deref().and_then(|o| o.as_dict());

    let mut walker = Walker {
        doc,
        budget,
        visits: 0,
        out: Vec::new(),
    };
    walker.walk_ops(&parsed.ops, resources, Matrix([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]), 0);
    Ok(walker.out)
}
