//! Drawing the knowledge map (change 171). Everything expensive — neighbours,
//! groups, positions, hulls, the order labels are tried in — arrives finished
//! in a `KnowledgeMap` from a background task. A frame transforms points by the
//! view and reads a label placement that is redone only when the view changes.

use crate::*;

/// Pan and zoom: a map point `p` is drawn at `rect.min + offset + p × scale`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MapView {
    pub(crate) scale: f32,
    pub(crate) offset: egui::Vec2,
    /// The scale 「全体を表示」 gave; zoom bounds and labels are relative to it.
    pub(crate) fitted_scale: f32,
    pub(crate) fitted: bool,
    /// The canvas size the view was last laid against.
    pub(crate) fitted_size: egui::Vec2,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: egui::Vec2::ZERO,
            fitted_scale: 1.0,
            fitted: false,
            fitted_size: egui::Vec2::ZERO,
        }
    }
}

const MAP_MARGIN: f32 = 24.0;

impl MapView {
    pub(crate) fn to_screen(self, rect: egui::Rect, point: [f32; 2]) -> egui::Pos2 {
        rect.min + self.offset + egui::vec2(point[0], point[1]) * self.scale
    }

    pub(crate) fn to_map(self, rect: egui::Rect, position: egui::Pos2) -> [f32; 2] {
        let local = (position - rect.min - self.offset) / self.scale;
        [local.x, local.y]
    }

    /// Zoom by `factor`, keeping the map point under `pointer` where it is.
    pub(crate) fn zoom_at(&mut self, rect: egui::Rect, pointer: egui::Pos2, factor: f32) {
        let under = self.to_map(rect, pointer);
        self.scale = (self.scale * factor).clamp(self.fitted_scale * 0.5, self.fitted_scale * 30.0);
        self.offset = pointer - rect.min - egui::vec2(under[0], under[1]) * self.scale;
    }

    /// Show the box `min`–`max` whole, centred, inside `rect`.
    pub(crate) fn frame(&mut self, rect: egui::Rect, min: [f32; 2], max: [f32; 2]) {
        let size = egui::vec2((max[0] - min[0]).max(1e-3), (max[1] - min[1]).max(1e-3));
        let room =
            (rect.size() - egui::vec2(MAP_MARGIN, MAP_MARGIN) * 2.0).max(egui::vec2(1.0, 1.0));
        self.scale = (room.x / size.x).min(room.y / size.y);
        let centre = egui::vec2((min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0);
        self.offset = rect.size() / 2.0 - centre * self.scale;
    }

    /// 「全体を表示」: frame every point, and make that the reference zoom.
    pub(crate) fn fit(&mut self, rect: egui::Rect, map: &KnowledgeMap) {
        self.frame(rect, map.min, map.max);
        self.fitted_scale = self.scale;
        self.fitted = true;
        self.fitted_size = rect.size();
    }

    /// The canvas changed size (a window resize, the narrow layout): refit,
    /// keeping how far the person had zoomed and the map point at the centre.
    pub(crate) fn resize(&mut self, rect: egui::Rect, map: &KnowledgeMap) {
        let old = egui::Rect::from_min_size(rect.min, self.fitted_size);
        let centre = self.to_map(old, old.center());
        let zoom = self.scale / self.fitted_scale;
        self.fit(rect, map);
        self.scale = self.fitted_scale * zoom;
        self.offset = rect.size() / 2.0 - egui::vec2(centre[0], centre[1]) * self.scale;
    }

    pub(crate) fn zoomed_in(&self) -> bool {
        self.scale >= self.fitted_scale * KNOWLEDGE_MAP_LABEL_ZOOM
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointShape {
    Diamond,
    Disc,
    Square,
    Triangle,
    Ring,
}

pub(crate) fn point_shape(kind: KnowledgeKind) -> PointShape {
    match kind {
        KnowledgeKind::Decision => PointShape::Diamond,
        KnowledgeKind::Cause => PointShape::Disc,
        KnowledgeKind::Procedure => PointShape::Square,
        KnowledgeKind::Pitfall => PointShape::Triangle,
        KnowledgeKind::Fact => PointShape::Ring,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PointStyle {
    pub(crate) shape: PointShape,
    pub(crate) radius: f32,
    pub(crate) opacity: f32,
    /// Below 0.3 freshness: drawn in `warning`, as the list's bar is.
    pub(crate) fading: bool,
    pub(crate) pinned: bool,
}

pub(crate) fn point_style(entry: &KnowledgeEntry, now: u64, dimmed: bool) -> PointStyle {
    let uses = (entry.reference_count + entry.recurrences) as f32;
    let freshness = entry.freshness(now) as f32;
    PointStyle {
        shape: point_shape(entry.kind),
        radius: (KNOWLEDGE_MAP_MIN_RADIUS + 1.5 * uses.sqrt()).min(KNOWLEDGE_MAP_MAX_RADIUS),
        opacity: if dimmed {
            KNOWLEDGE_MAP_DIMMED_OPACITY
        } else {
            freshness.max(KNOWLEDGE_MAP_MIN_OPACITY)
        },
        fading: freshness < 0.3,
        pinned: entry.pinned,
    }
}

/// The hover text: the title, then kind, freshness, and use.
pub(crate) fn point_tooltip(entry: &KnowledgeEntry, now: u64) -> (String, String) {
    (
        entry.title.clone(),
        tf!(
            "{kind} ・ 鮮度 {p0}% ・ 参照 {n}",
            kind = entry.kind.label(),
            p0 = (entry.freshness(now) * 100.0).round() as u32,
            n = entry.reference_count
        ),
    )
}

pub(crate) fn paint_point_shape(
    painter: &egui::Painter,
    centre: egui::Pos2,
    shape: PointShape,
    radius: f32,
    colour: egui::Color32,
) {
    let stroke = egui::Stroke::NONE;
    match shape {
        PointShape::Disc => {
            painter.circle_filled(centre, radius, colour);
        }
        PointShape::Ring => {
            painter.circle_stroke(centre, radius - 0.75, egui::Stroke::new(1.5, colour));
        }
        PointShape::Square => {
            let half = radius * 0.85;
            painter.rect_filled(
                egui::Rect::from_center_size(centre, egui::vec2(half, half) * 2.0),
                1.0,
                colour,
            );
        }
        PointShape::Diamond => {
            let r = radius * 1.15;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    centre + egui::vec2(0.0, -r),
                    centre + egui::vec2(r, 0.0),
                    centre + egui::vec2(0.0, r),
                    centre + egui::vec2(-r, 0.0),
                ],
                colour,
                stroke,
            ));
        }
        PointShape::Triangle => {
            let r = radius * 1.2;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    centre + egui::vec2(0.0, -r),
                    centre + egui::vec2(r * 0.87, r * 0.5),
                    centre + egui::vec2(-r * 0.87, r * 0.5),
                ],
                colour,
                stroke,
            ));
        }
    }
}

/// What the cached label placement was made for. Positions are compared by
/// bits, so any pan, zoom, resize, or new result invalidates it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LabelKey {
    map: usize,
    scale: u32,
    offset: [u32; 2],
    rect: [u32; 4],
}

impl LabelKey {
    pub(crate) fn new(map: &Arc<KnowledgeMap>, view: &MapView, rect: egui::Rect) -> Self {
        Self {
            map: Arc::as_ptr(map) as usize,
            scale: view.scale.to_bits(),
            offset: [view.offset.x.to_bits(), view.offset.y.to_bits()],
            rect: [
                rect.min.x.to_bits(),
                rect.min.y.to_bits(),
                rect.max.x.to_bits(),
                rect.max.y.to_bits(),
            ],
        }
    }
}

/// Screen-space geometry that depends on the view: group outlines, the group
/// names' boxes, and the entry titles that fit without overlapping.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct LabelPlacement {
    pub(crate) hulls: Vec<Vec<egui::Pos2>>,
    pub(crate) groups: Vec<egui::Rect>,
    /// Point index and the box its title is drawn in.
    pub(crate) entries: Vec<(usize, egui::Rect)>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LabelCache {
    key: Option<LabelKey>,
    placement: Arc<LabelPlacement>,
}

impl LabelCache {
    /// The placement for `key`, recomputed by `place` only when `key` differs
    /// from the one it was made for.
    pub(crate) fn get_or_place(
        &mut self,
        key: LabelKey,
        place: impl FnOnce() -> LabelPlacement,
    ) -> Arc<LabelPlacement> {
        if self.key != Some(key) {
            self.placement = Arc::new(place());
            self.key = Some(key);
        }
        self.placement.clone()
    }
}

/// Place group outlines and names, then — zoomed in — entry titles in the
/// map's label order, skipping any that would leave the map or overlap a name
/// already placed, up to `KNOWLEDGE_MAP_MAX_LABELS`. `measure` gives a text's
/// size, so this runs without a font in tests.
pub(crate) fn place_labels(
    map: &KnowledgeMap,
    entries: &[KnowledgeEntry],
    view: &MapView,
    rect: egui::Rect,
    measure: &dyn Fn(&str) -> egui::Vec2,
) -> LabelPlacement {
    let pad = KNOWLEDGE_MAP_MAX_RADIUS + 4.0;
    let hulls: Vec<Vec<egui::Pos2>> = map
        .groups
        .iter()
        .map(|group| {
            let points: Vec<egui::Pos2> = group
                .hull
                .iter()
                .map(|point| view.to_screen(rect, *point))
                .collect();
            let count = points.len().max(1) as f32;
            let centre = points
                .iter()
                .fold(egui::Vec2::ZERO, |sum, point| sum + point.to_vec2())
                / count;
            points
                .iter()
                .map(|point| {
                    let away = point.to_vec2() - centre;
                    let length = away.length();
                    if length > 0.0 {
                        *point + away / length * pad
                    } else {
                        *point
                    }
                })
                .collect()
        })
        .collect();
    let mut groups: Vec<egui::Rect> = Vec::with_capacity(map.groups.len());
    for (group, hull) in map.groups.iter().zip(&hulls) {
        let size = measure(&group.name);
        let mut outline = egui::Rect::NOTHING;
        for point in hull {
            outline.extend_with(*point);
        }
        if hull.is_empty() {
            outline = egui::Rect::from_center_size(rect.center(), egui::Vec2::ZERO);
        }
        let middle = outline.center().x - size.x / 2.0;
        let beside = outline.center().y - size.y / 2.0;
        // Above the outline first; below, right, or left of it when the map's
        // edge would cut the name off or an earlier name is already there.
        let candidates = [
            egui::pos2(middle, outline.top() - size.y - 2.0),
            egui::pos2(middle, outline.bottom() + 2.0),
            egui::pos2(outline.right() + 4.0, beside),
            egui::pos2(outline.left() - size.x - 4.0, beside),
        ]
        .map(|corner| egui::Rect::from_min_size(corner, size));
        let fits = |candidate: &egui::Rect| {
            candidate.top() >= rect.top() && candidate.bottom() <= rect.bottom()
        };
        let chosen = candidates
            .iter()
            .find(|candidate| {
                fits(candidate) && !groups.iter().any(|other| other.intersects(**candidate))
            })
            .or_else(|| candidates.iter().find(|candidate| fits(candidate)))
            .unwrap_or(&candidates[0]);
        groups.push(*chosen);
    }
    let mut placed: Vec<(usize, egui::Rect)> = Vec::new();
    if view.zoomed_in() {
        let inside = rect.shrink(2.0);
        for index in &map.label_order {
            if placed.len() >= KNOWLEDGE_MAP_MAX_LABELS {
                break;
            }
            let Some(title) = entries.get(*index).map(|entry| entry.title.as_str()) else {
                continue;
            };
            let anchor = view.to_screen(rect, map.points[*index]);
            if !inside.contains(anchor) {
                continue;
            }
            let size = measure(title);
            let candidate = egui::Rect::from_min_size(
                anchor + egui::vec2(KNOWLEDGE_MAP_MAX_RADIUS, -size.y / 2.0),
                size,
            );
            if !inside.contains_rect(candidate)
                || groups.iter().any(|group| group.intersects(candidate))
                || placed.iter().any(|(_, other)| other.intersects(candidate))
            {
                continue;
            }
            placed.push((*index, candidate));
        }
    }
    LabelPlacement {
        hulls,
        groups,
        entries: placed,
    }
}

/// What the person did on the map this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct MapResponse {
    pub(crate) hovered: Option<usize>,
    pub(crate) clicked: Option<usize>,
    pub(crate) clicked_empty: bool,
}

/// The state a frame draws from, besides the map itself.
pub(crate) struct MapFocus<'a> {
    /// Per point: dimmed by a search or a kind filter.
    pub(crate) dimmed: &'a [bool],
    pub(crate) chosen: Option<usize>,
    pub(crate) group: Option<MapGroupChoice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapGroupChoice {
    Group(usize),
    Ungrouped,
}

impl MapGroupChoice {
    pub(crate) fn holds(self, map: &KnowledgeMap, index: usize) -> bool {
        match self {
            Self::Group(group) => map.group_of[index] == Some(group),
            Self::Ungrouped => map.group_of[index].is_none(),
        }
    }
}

const GROUP_FILL: f32 = 0.08;
const QUIET_LINE: f32 = 0.35;

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_knowledge_map(
    ui: &mut egui::Ui,
    palette: &Palette,
    map: &Arc<KnowledgeMap>,
    entries: &[KnowledgeEntry],
    size: egui::Vec2,
    view: &mut MapView,
    labels: &mut LabelCache,
    focus: &MapFocus<'_>,
    now: u64,
) -> MapResponse {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect(
        rect,
        6.0,
        palette.inset,
        egui::Stroke::new(1.0, palette.border_subtle),
        egui::StrokeKind::Inside,
    );
    if !view.fitted {
        view.fit(rect, map);
    } else if view.fitted_size != rect.size() {
        view.resize(rect, map);
    }
    if response.dragged() {
        view.offset += response.drag_delta();
    }
    if response.hovered() {
        let (scroll, zoom, pointer) = ui.input(|input| {
            (
                input.smooth_scroll_delta.y,
                input.zoom_delta(),
                input.pointer.hover_pos(),
            )
        });
        let factor = zoom * (scroll * 0.0025).exp();
        if let Some(pointer) = pointer.filter(|_| (factor - 1.0).abs() > 1e-4) {
            view.zoom_at(rect, pointer, factor);
        }
        // The map owns the wheel while the pointer is on it; the page must not
        // scroll underneath.
        ui.input_mut(|input| input.smooth_scroll_delta = egui::Vec2::ZERO);
    }

    let placement = labels.get_or_place(LabelKey::new(map, view, rect), || {
        let fonts = ui.fonts(|fonts| fonts.clone());
        place_labels(map, entries, view, rect, &|text: &str| {
            fonts
                .layout_no_wrap(
                    text.to_owned(),
                    egui::FontId::proportional(12.0),
                    palette.text,
                )
                .size()
        })
    });

    // Hover: the nearest point within reach of the pointer.
    let hovered = response.hover_pos().and_then(|pointer| {
        let mut best: Option<(usize, f32)> = None;
        for (index, point) in map.points.iter().enumerate() {
            let distance = view.to_screen(rect, *point).distance(pointer);
            let reach = KNOWLEDGE_MAP_MAX_RADIUS + 3.0;
            if distance <= reach && best.is_none_or(|(_, nearest)| distance < nearest) {
                best = Some((index, distance));
            }
        }
        best.map(|(index, _)| index)
    });
    let spotlight = hovered.or(focus.chosen);

    for (index, hull) in placement.hulls.iter().enumerate() {
        if hull.len() < 3 {
            continue;
        }
        let faded = focus
            .group
            .is_some_and(|group| group != MapGroupChoice::Group(index));
        let fill = palette
            .accent
            .gamma_multiply(if faded { GROUP_FILL / 3.0 } else { GROUP_FILL });
        painter.add(egui::Shape::convex_polygon(
            hull.clone(),
            fill,
            egui::Stroke::new(1.0, palette.border_subtle),
        ));
    }

    for (a, b) in &map.edges {
        let from = view.to_screen(rect, map.points[*a]);
        let to = view.to_screen(rect, map.points[*b]);
        if !rect.intersects(egui::Rect::from_two_pos(from, to)) {
            continue;
        }
        let lit = spotlight.is_some_and(|point| point == *a || point == *b);
        let stroke = if lit {
            egui::Stroke::new(1.5, palette.accent)
        } else if spotlight.is_some() || focus.dimmed[*a] || focus.dimmed[*b] {
            egui::Stroke::new(1.0, palette.border_subtle.gamma_multiply(QUIET_LINE))
        } else {
            egui::Stroke::new(1.0, palette.border_subtle)
        };
        painter.line_segment([from, to], stroke);
    }

    let visible = rect.expand(KNOWLEDGE_MAP_MAX_RADIUS * 2.0);
    for (index, entry) in entries.iter().enumerate() {
        let Some(point) = map.points.get(index) else {
            break;
        };
        let centre = view.to_screen(rect, *point);
        if !visible.contains(centre) {
            continue;
        }
        let outside_group = focus.group.is_some_and(|group| !group.holds(map, index));
        let near_spotlight =
            spotlight.is_none_or(|lit| lit == index || map.linked[lit].contains(&index));
        let style = point_style(entry, now, focus.dimmed[index] || outside_group);
        let mut opacity = style.opacity;
        if !near_spotlight {
            opacity = opacity.min(0.5);
        }
        let ink = if style.fading {
            palette.warning
        } else {
            palette.accent
        };
        paint_point_shape(
            &painter,
            centre,
            style.shape,
            style.radius,
            ink.gamma_multiply(opacity),
        );
        if style.pinned {
            painter.circle_stroke(
                centre,
                style.radius + 2.5,
                egui::Stroke::new(1.2, palette.text_strong.gamma_multiply(opacity)),
            );
        }
        if focus.chosen == Some(index) || hovered == Some(index) {
            painter.circle_stroke(
                centre,
                style.radius + 4.0,
                egui::Stroke::new(2.0, palette.text_strong),
            );
        }
    }

    for (group, area) in map.groups.iter().zip(&placement.groups) {
        painter.text(
            area.left_top(),
            egui::Align2::LEFT_TOP,
            &group.name,
            egui::FontId::proportional(12.0),
            palette.text_muted,
        );
    }
    for (index, area) in &placement.entries {
        painter.text(
            area.left_top(),
            egui::Align2::LEFT_TOP,
            &entries[*index].title,
            egui::FontId::proportional(12.0),
            palette.text,
        );
    }

    let corner = egui::Rect::from_min_size(
        egui::pos2(rect.max.x - CONTROL_HEIGHT_SMALL - 6.0, rect.min.y + 6.0),
        egui::vec2(CONTROL_HEIGHT_SMALL, CONTROL_HEIGHT_SMALL),
    );
    let mut corner_ui = ui.new_child(egui::UiBuilder::new().max_rect(corner));
    if small_icon_button(&mut corner_ui, palette, ICON_FIT, tr("全体を表示")).clicked() {
        view.fit(rect, map);
    }

    if let Some(index) = hovered {
        let (title, line) = point_tooltip(&entries[index], now);
        response.clone().on_hover_ui_at_pointer(|ui| {
            ui.label(RichText::new(title).color(palette.text_strong));
            ui.label(RichText::new(line).size(12.0).color(palette.text_muted));
        });
    }
    MapResponse {
        hovered,
        clicked: if response.clicked() { hovered } else { None },
        clicked_empty: response.clicked() && hovered.is_none(),
    }
}

/// The legend under the map: the five shapes, then what size, fill, and the
/// ring mean.
pub(crate) fn knowledge_map_legend(ui: &mut egui::Ui, palette: &Palette) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_XS;
        for kind in KnowledgeKind::ALL {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
            paint_point_shape(
                ui.painter(),
                rect.center(),
                point_shape(kind),
                4.5,
                palette.accent,
            );
            ui.label(
                RichText::new(kind.label())
                    .size(12.0)
                    .color(palette.text_muted),
            );
            ui.add_space(SPACE_SM);
        }
        ui.label(
            RichText::new(tr("・ 大きさ: 参照回数 ・ 濃さ: 鮮度 ・"))
                .size(12.0)
                .color(palette.text_muted),
        );
        let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 12.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 3.5, palette.accent);
        ui.painter().circle_stroke(
            rect.center(),
            6.0,
            egui::Stroke::new(1.2, palette.text_strong),
        );
        ui.label(
            RichText::new(tr("固定"))
                .size(12.0)
                .color(palette.text_muted),
        );
    });
}
