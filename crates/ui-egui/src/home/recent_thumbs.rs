//! Off-thread first-page thumbnail cache for Recent PDF cards on Home.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use egui::{Color32, CornerRadius, Rect, Stroke, TextureHandle, Vec2};

struct PendingResult {
    path: String,
    image: Option<([u32; 2], pagic_render::Pixels)>,
}

#[derive(Clone, Default)]
pub(super) struct RecentThumbCache {
    inner: Arc<Mutex<CacheInner>>,
}

#[derive(Default)]
struct CacheInner {
    /// Cached GPU textures (or `None` when rendering failed / file unavailable).
    textures: HashMap<String, Option<TextureHandle>>,
    /// Paths currently queued or rendering on a background thread.
    in_flight: HashMap<String, ()>,
    /// Completed renders waiting to be uploaded to GPU textures on the UI thread.
    ready: Vec<PendingResult>,
}

impl RecentThumbCache {
    /// Upload any finished background renders to egui textures and return the cached texture for
    /// `path` if available, or kick off a background render if not yet requested.
    pub(super) fn get_or_request(&self, ctx: &egui::Context, path: &str) -> Option<TextureHandle> {
        let mut guard = self.inner.lock().ok()?;
        let ready: Vec<PendingResult> = std::mem::take(&mut guard.ready);
        for res in ready {
            guard.in_flight.remove(&res.path);
            let tex = res.image.and_then(|([w, h], pixels)| {
                let (w, h) = (w as usize, h as usize);
                if w == 0 || h == 0 || pixels.len() != w.saturating_mul(h).saturating_mul(4) {
                    return None;
                }
                let img = crate::canvas::texture_image([w, h], pixels);
                Some(ctx.load_texture(format!("recent-thumb-{}", res.path), img, egui::TextureOptions::LINEAR))
            });
            guard.textures.insert(res.path, tex);
        }

        if let Some(cached) = guard.textures.get(path) {
            return cached.clone();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if !guard.in_flight.contains_key(path) {
            guard.in_flight.insert(path.to_owned(), ());
            let inner = Arc::clone(&self.inner);
            let path_owned = path.to_owned();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                let image = render_first_page(&path_owned);
                if let Ok(mut g) = inner.lock() {
                    g.ready.push(PendingResult { path: path_owned, image });
                }
                ctx.request_repaint();
            });
        }

        None
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_first_page(path: &str) -> Option<([u32; 2], pagic_render::Pixels)> {
    let bytes = std::fs::read(path).ok()?;
    let pool = pagic_render::RenderPool::new_inline(Arc::new(bytes), pagic_render::RenderConfig::default());
    pool.set_queue(vec![pagic_render::RenderRequest {
        page: 0,
        scale: 0.45,
        tag: 1,
        ..Default::default()
    }]);
    let rendered = pool.try_recv()?;
    if rendered.error.is_some() || rendered.width == 0 || rendered.height == 0 {
        return None;
    }
    Some(([rendered.width, rendered.height], rendered.rgba))
}

/// Paint a PDF thumbnail texture fitted inside `preview_rect`, preserving aspect ratio with a subtle border.
pub(super) fn paint_fitted(ui: &egui::Ui, preview_rect: Rect, tex: &TextureHandle, border_color: Color32) {
    let inner = preview_rect.shrink(6.0);
    let tex_size = tex.size_vec2();
    if tex_size.x <= 0.0 || tex_size.y <= 0.0 || inner.width() <= 0.0 || inner.height() <= 0.0 {
        return;
    }
    let scale = (inner.width() / tex_size.x).min(inner.height() / tex_size.y);
    let draw_size = Vec2::new((tex_size.x * scale).round(), (tex_size.y * scale).round());
    let draw_rect = Rect::from_center_size(inner.center(), draw_size);
    ui.painter().image(
        tex.id(),
        draw_rect,
        Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    ui.painter().rect_stroke(draw_rect, CornerRadius::same(2), Stroke::new(1.0, border_color), egui::StrokeKind::Outside);
}
