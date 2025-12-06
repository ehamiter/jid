use gpui::*;
use std::ops::Range;
use unicode_segmentation::*;

use crate::actions::ToggleFocusMode;
use crate::theme::Theme;

pub enum EditorEvent {
    Modified,
    FocusModeChanged(bool),
}

impl EventEmitter<EditorEvent> for EditorView {}

const PADDING: f32 = 48.0;
const LINE_HEIGHT: f32 = 38.0;
const FONT_SIZE: f32 = 21.0;

pub struct EditorView {
    focus_handle: FocusHandle,
    content: String,
    selected_range: Range<usize>,
    marked_range: Option<Range<usize>>,
    last_layout: Option<WrappedLayout>,
    last_content_bounds: Option<Bounds<Pixels>>,
    theme: Theme,
    modified: bool,
    focus_mode: bool,
    scroll_y: Pixels,
    pending_scroll_to_cursor: bool,
}

#[derive(Clone)]
struct WrappedLayout {
    lines: Vec<ShapedLine>,
    line_ranges: Vec<Range<usize>>,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>, theme: Theme, focus_mode: bool) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: String::new(),
            selected_range: 0..0,
            marked_range: None,
            last_layout: None,
            last_content_bounds: None,
            theme,
            modified: false,
            focus_mode,
            scroll_y: px(0.0),
            pending_scroll_to_cursor: false,
        }
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        cx.notify();
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    pub fn toggle_focus_mode(&mut self, _: &ToggleFocusMode, _window: &mut Window, cx: &mut Context<Self>) {
        self.focus_mode = !self.focus_mode;
        cx.emit(EditorEvent::FocusModeChanged(self.focus_mode));
        cx.notify();
    }

    fn on_scroll(&mut self, event: &ScrollWheelEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let delta = match event.delta {
            ScrollDelta::Lines(delta) => delta.y * px(LINE_HEIGHT),
            ScrollDelta::Pixels(delta) => delta.y,
        };
        
        self.scroll_y = (self.scroll_y - delta).max(px(0.0));
        
        if let (Some(layout), Some(bounds)) = (&self.last_layout, &self.last_content_bounds) {
            let line_height = px(LINE_HEIGHT);
            let total_height = line_height * layout.line_ranges.len() as f32;
            let viewport_height = bounds.size.height;
            let max_scroll = (total_height - viewport_height).max(px(0.0));
            self.scroll_y = self.scroll_y.min(max_scroll);
        }
        
        cx.notify();
    }

    fn left(&mut self, _: &Left, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.selected_range.start), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn right(&mut self, _: &Right, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn up(&mut self, _: &Up, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(layout) = &self.last_layout {
            let cursor = self.selected_range.start;
            if let Some((line_idx, _)) = self.line_for_offset(cursor) {
                if line_idx > 0 {
                    let col = cursor - layout.line_ranges[line_idx].start;
                    let prev_range = &layout.line_ranges[line_idx - 1];
                    let line_len = prev_range.end - prev_range.start;
                    let new_pos = prev_range.start + col.min(line_len);
                    self.move_to(new_pos, cx);
                    return;
                }
            }
        }
        self.move_to(0, cx);
    }

    fn down(&mut self, _: &Down, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(layout) = &self.last_layout {
            let cursor = self.selected_range.start;
            if let Some((line_idx, _)) = self.line_for_offset(cursor) {
                if line_idx + 1 < layout.line_ranges.len() {
                    let col = cursor - layout.line_ranges[line_idx].start;
                    let next_range = &layout.line_ranges[line_idx + 1];
                    let line_len = next_range.end - next_range.start;
                    let new_pos = next_range.start + col.min(line_len);
                    self.move_to(new_pos, cx);
                    return;
                }
            }
        }
        self.move_to(self.content.len(), cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _window: &mut Window, cx: &mut Context<Self>) {
        let new_end = self.previous_boundary(self.selected_range.end);
        self.select_to(new_end, cx);
    }

    fn select_right(&mut self, _: &SelectRight, _window: &mut Window, cx: &mut Context<Self>) {
        let new_end = self.next_boundary(self.selected_range.end);
        self.select_to(new_end, cx);
    }

    fn select_up(&mut self, _: &SelectUp, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(layout) = &self.last_layout.clone() {
            let cursor = self.selected_range.end;
            if let Some((line_idx, _)) = self.line_for_offset(cursor) {
                if line_idx > 0 {
                    let col = cursor - layout.line_ranges[line_idx].start;
                    let prev_range = &layout.line_ranges[line_idx - 1];
                    let line_len = prev_range.end - prev_range.start;
                    let new_pos = prev_range.start + col.min(line_len);
                    self.select_to(new_pos, cx);
                    return;
                }
            }
        }
        self.select_to(0, cx);
    }

    fn select_down(&mut self, _: &SelectDown, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(layout) = &self.last_layout.clone() {
            let cursor = self.selected_range.end;
            if let Some((line_idx, _)) = self.line_for_offset(cursor) {
                if line_idx + 1 < layout.line_ranges.len() {
                    let col = cursor - layout.line_ranges[line_idx].start;
                    let next_range = &layout.line_ranges[line_idx + 1];
                    let line_len = next_range.end - next_range.start;
                    let new_pos = next_range.start + col.min(line_len);
                    self.select_to(new_pos, cx);
                    return;
                }
            }
        }
        self.select_to(self.content.len(), cx);
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = self.selected_range.start..offset;
        if self.selected_range.end < self.selected_range.start {
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn line_for_offset(&self, offset: usize) -> Option<(usize, &Range<usize>)> {
        self.last_layout.as_ref().and_then(|layout| {
            layout.line_ranges.iter().enumerate().find(|(_, range)| {
                offset >= range.start && offset <= range.end
            })
        })
    }

    fn backspace(&mut self, _: &Backspace, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.selected_range.start);
            self.selected_range = prev..self.selected_range.start;
        }
        self.replace_text(&self.selected_range.clone(), "", cx);
    }

    fn delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.selected_range.end);
            self.selected_range = self.selected_range.end..next;
        }
        self.replace_text(&self.selected_range.clone(), "", cx);
    }

    fn newline(&mut self, _: &Newline, _window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text(&self.selected_range.clone(), "\n", cx);
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_range = 0..self.content.len();
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text(&self.selected_range.clone(), "", cx);
        }
    }

    fn paste(&mut self, _: &Paste, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text(&self.selected_range.clone(), &text, cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.pending_scroll_to_cursor = true;
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }
        if offset > 0 && self.content.as_bytes().get(offset - 1) == Some(&b'\n') {
            return offset - 1;
        }
        self.content[..offset]
            .grapheme_indices(true)
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        if offset >= self.content.len() {
            return self.content.len();
        }
        if self.content.as_bytes().get(offset) == Some(&b'\n') {
            return offset + 1;
        }
        self.content[offset..]
            .grapheme_indices(true)
            .nth(1)
            .map(|(i, _)| offset + i)
            .unwrap_or(self.content.len())
    }

    fn replace_text(&mut self, range: &Range<usize>, new_text: &str, cx: &mut Context<Self>) {
        self.content = format!(
            "{}{}{}",
            &self.content[..range.start],
            new_text,
            &self.content[range.end..]
        );
        let new_cursor = range.start + new_text.len();
        self.selected_range = new_cursor..new_cursor;
        self.marked_range = None;
        self.modified = true;
        self.pending_scroll_to_cursor = true;
        cx.notify();
        cx.emit(EditorEvent::Modified);
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let offset = self.index_for_position(event.position);
        self.move_to(offset, cx);
        window.focus(&self.focus_handle);
    }

    fn index_for_position(&self, position: Point<Pixels>) -> usize {
        let Some(content_bounds) = self.last_content_bounds.as_ref() else { return 0 };
        let Some(layout) = self.last_layout.as_ref() else { return 0 };

        if layout.lines.is_empty() || layout.line_ranges.is_empty() {
            return 0;
        }

        let local_y = position.y - content_bounds.top() + self.scroll_y;
        let line_height = px(LINE_HEIGHT);
        let line_idx = ((local_y / line_height).floor() as usize)
            .max(0)
            .min(layout.lines.len().saturating_sub(1));

        let local_x = (position.x - content_bounds.left()).max(px(0.0));
        let line = &layout.lines[line_idx];
        let range = &layout.line_ranges[line_idx];
        let char_offset = line.closest_index_for_x(local_x);
        let line_len = range.end - range.start;
        range.start + char_offset.min(line_len)
    }
}

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        Some(self.content.clone())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.selected_range.start..self.selected_range.end,
            reversed: false,
        })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .or(self.marked_range.take())
            .unwrap_or(self.selected_range.clone());

        self.content = format!(
            "{}{}{}",
            &self.content[..range.start],
            new_text,
            &self.content[range.end..]
        );

        let new_cursor = range.start + new_text.len();
        self.selected_range = new_cursor..new_cursor;
        self.modified = true;
        cx.notify();
        cx.emit(EditorEvent::Modified);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .or(self.marked_range.take())
            .unwrap_or(self.selected_range.clone());

        self.content = format!(
            "{}{}{}",
            &self.content[..range.start],
            new_text,
            &self.content[range.end..]
        );

        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }

        self.selected_range = new_selected_range_utf16
            .map(|r| range.start + r.start..range.start + r.end)
            .unwrap_or_else(|| {
                let pos = range.start + new_text.len();
                pos..pos
            });

        self.modified = true;
        cx.notify();
        cx.emit(EditorEvent::Modified);
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        self.last_content_bounds
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.index_for_position(point))
    }
}

actions!(
    editor,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        Newline,
        SelectAll,
        Copy,
        Cut,
        Paste,
    ]
);

impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = self.content.clone();
        let selected_range = self.selected_range.clone();
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let entity = cx.entity().clone();

        let cursor_pos = self.selected_range.start;

        let placeholder_visible = content.is_empty();
        let focus_mode = self.focus_mode;
        let scroll_y = self.scroll_y;
        let pending_scroll_to_cursor = self.pending_scroll_to_cursor;
        self.pending_scroll_to_cursor = false;

        div()
            .id("editor")
            .key_context("Editor")
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::toggle_focus_mode))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .size_full()
            .cursor(CursorStyle::IBeam)
            .child(
                EditorElement {
                    content,
                    selected_range,
                    is_focused,
                    placeholder_visible,
                    cursor_pos,
                    focus_mode,
                    scroll_y,
                    pending_scroll_to_cursor,
                    entity,
                    theme: self.theme.clone(),
                }
            )
    }
}

struct EditorElement {
    content: String,
    selected_range: Range<usize>,
    is_focused: bool,
    placeholder_visible: bool,
    cursor_pos: usize,
    focus_mode: bool,
    scroll_y: Pixels,
    pending_scroll_to_cursor: bool,
    entity: Entity<EditorView>,
    theme: Theme,
}

impl IntoElement for EditorElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

struct EditorPrepaintState {
    lines: Vec<ShapedLine>,
    line_ranges: Vec<Range<usize>>,
    scroll_y: Pixels,
    cursor_pos: Option<(Pixels, Pixels)>,
    selections: Vec<Bounds<Pixels>>,
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = EditorPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some("editor-element".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = px(800.0).into();
        style.size.height = relative(1.0).into();
        style.padding = Edges {
            top: px(PADDING).into(),
            right: px(PADDING).into(),
            bottom: px(PADDING).into(),
            left: px(PADDING).into(),
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let font_size = px(FONT_SIZE);
        let line_height = px(LINE_HEIGHT);
        let padding = px(PADDING);
        let available_width = bounds.size.width - padding * 2.0;

        let style = window.text_style();
        let font = style.font();
        let mut visual_lines: Vec<ShapedLine> = Vec::new();
        let mut visual_ranges: Vec<Range<usize>> = Vec::new();

        if self.placeholder_visible {
            let text: SharedString = "Start writing...".into();
            let run = TextRun {
                len: text.len(),
                font,
                color: self.theme.muted,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let line = window.text_system().shape_line(text, font_size, &[run], None);
            visual_lines.push(line);
            visual_ranges.push(0..0);
        } else {
            // First pass: compute visual line ranges without shaping
            let mut temp_ranges: Vec<Range<usize>> = Vec::new();
            let mut logical_start = 0usize;
            for logical_line in self.content.split('\n') {
                let logical_len = logical_line.len();
                let logical_end = logical_start + logical_len;

                if logical_line.is_empty() {
                    temp_ranges.push(logical_start..logical_start);
                } else {
                    let mut start = 0usize;
                    while start < logical_len {
                        let mut end = logical_len;
                        let mut last_break = start;

                        for (i, c) in logical_line[start..].char_indices() {
                            let pos = start + i + c.len_utf8();
                            if c.is_whitespace() || c == '-' {
                                last_break = pos;
                            }

                            let slice = &logical_line[start..pos];
                            let text: SharedString = slice.to_string().into();
                            let run = TextRun {
                                len: text.len(),
                                font: font.clone(),
                                color: self.theme.foreground,
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            };
                            let shaped = window.text_system().shape_line(text, font_size, &[run], None);
                            if shaped.width > available_width && pos > start + 1 {
                                end = if last_break > start { last_break } else { pos - c.len_utf8() };
                                break;
                            }
                        }

                        let break_at = end.min(logical_len);
                        if break_at == start {
                            break;
                        }

                        temp_ranges.push((logical_start + start)..(logical_start + break_at));

                        start = break_at;
                        while start < logical_len && logical_line.as_bytes().get(start) == Some(&b' ') {
                            start += 1;
                        }
                    }
                }

                logical_start = logical_end + 1;
            }

            // Determine which visual line the cursor is on
            let current_visual_line = temp_ranges
                .iter()
                .position(|r| self.cursor_pos >= r.start && self.cursor_pos <= r.end);

            // Second pass: shape lines with correct colors
            logical_start = 0usize;
            let mut visual_line_idx = 0usize;
            for logical_line in self.content.split('\n') {
                let logical_len = logical_line.len();
                let logical_end = logical_start + logical_len;

                if logical_line.is_empty() {
                    let is_current = current_visual_line == Some(visual_line_idx);
                    let color = if self.focus_mode && !is_current {
                        self.theme.muted
                    } else {
                        self.theme.foreground
                    };

                    let text: SharedString = "".into();
                    let run = TextRun {
                        len: 0,
                        font: font.clone(),
                        color,
                        background_color: None,
                        underline: None,
                        strikethrough: None,
                    };
                    let line = window.text_system().shape_line(text, font_size, &[run], None);
                    visual_lines.push(line);
                    visual_ranges.push(logical_start..logical_start);
                    visual_line_idx += 1;
                } else {
                    let mut start = 0usize;
                    while start < logical_len {
                        let mut end = logical_len;
                        let mut last_break = start;

                        for (i, c) in logical_line[start..].char_indices() {
                            let pos = start + i + c.len_utf8();
                            if c.is_whitespace() || c == '-' {
                                last_break = pos;
                            }

                            let slice = &logical_line[start..pos];
                            let text: SharedString = slice.to_string().into();
                            let run = TextRun {
                                len: text.len(),
                                font: font.clone(),
                                color: self.theme.foreground,
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            };
                            let shaped = window.text_system().shape_line(text, font_size, &[run], None);
                            if shaped.width > available_width && pos > start + 1 {
                                end = if last_break > start { last_break } else { pos - c.len_utf8() };
                                break;
                            }
                        }

                        let break_at = end.min(logical_len);
                        if break_at == start {
                            break;
                        }

                        let is_current = current_visual_line == Some(visual_line_idx);
                        let color = if self.focus_mode && !is_current {
                            self.theme.muted
                        } else {
                            self.theme.foreground
                        };

                        let slice = &logical_line[start..break_at];
                        let text: SharedString = slice.to_string().into();
                        let run = TextRun {
                            len: text.len(),
                            font: font.clone(),
                            color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        let shaped = window.text_system().shape_line(text, font_size, &[run], None);
                        visual_lines.push(shaped);
                        visual_ranges.push((logical_start + start)..(logical_start + break_at));

                        visual_line_idx += 1;
                        start = break_at;
                        while start < logical_len && logical_line.as_bytes().get(start) == Some(&b' ') {
                            start += 1;
                        }
                    }
                }

                logical_start = logical_end + 1;
            }
        }

        let content_origin = point(bounds.left() + padding, bounds.top() + padding);
        let viewport_height = bounds.size.height - padding * 2.0;

        let mut scroll_y = self.scroll_y;
        if self.pending_scroll_to_cursor && !self.placeholder_visible {
            let cursor = self.selected_range.start;
            if let Some(line_idx) = visual_ranges.iter().position(|r| cursor >= r.start && cursor <= r.end) {
                let cursor_top = line_height * line_idx as f32;
                let cursor_bottom = cursor_top + line_height;

                if cursor_top < scroll_y {
                    scroll_y = cursor_top;
                } else if cursor_bottom > scroll_y + viewport_height {
                    scroll_y = cursor_bottom - viewport_height;
                }

                let total_height = line_height * visual_ranges.len() as f32;
                let max_scroll = (total_height - viewport_height).max(px(0.0));
                scroll_y = scroll_y.clamp(px(0.0), max_scroll);
            }
        }

        let mut selections = Vec::new();
        let cursor_pos;

        if !self.placeholder_visible && self.is_focused {
            if self.selected_range.is_empty() {
                let cursor_offset = self.selected_range.start;
                let mut cursor_line = 0;
                let mut cursor_col = 0;

                for (i, range) in visual_ranges.iter().enumerate() {
                    if cursor_offset >= range.start && cursor_offset <= range.end {
                        cursor_line = i;
                        cursor_col = cursor_offset - range.start;
                        break;
                    }
                }

                let x = if cursor_line < visual_lines.len() {
                    visual_lines[cursor_line].x_for_index(cursor_col)
                } else {
                    px(0.0)
                };

                let y = line_height * cursor_line as f32 - scroll_y;
                cursor_pos = Some((x, y));
            } else {
                cursor_pos = None;

                for (line_idx, line_range) in visual_ranges.iter().enumerate() {
                    let sel_start = self.selected_range.start;
                    let sel_end = self.selected_range.end;

                    if sel_end <= line_range.start || sel_start > line_range.end {
                        continue;
                    }

                    let start_in_line = if sel_start <= line_range.start {
                        0
                    } else {
                        sel_start - line_range.start
                    };

                    let end_in_line = if sel_end > line_range.end {
                        line_range.end - line_range.start
                    } else {
                        sel_end - line_range.start
                    };

                    if line_idx < visual_lines.len() {
                        let x1 = visual_lines[line_idx].x_for_index(start_in_line);
                        let x2 = visual_lines[line_idx].x_for_index(end_in_line);
                        let y = line_height * line_idx as f32 - scroll_y;

                        let sel_bounds = Bounds::new(
                            point(content_origin.x + x1, content_origin.y + y),
                            size(x2 - x1, line_height),
                        );
                        selections.push(sel_bounds);
                    }
                }
            }
        } else {
            cursor_pos = None;
        }

        EditorPrepaintState {
            lines: visual_lines,
            line_ranges: visual_ranges,
            scroll_y,
            cursor_pos,
            selections,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let line_height = px(LINE_HEIGHT);
        let padding = px(PADDING);
        let content_origin = point(bounds.left() + padding, bounds.top() + padding);
        let viewport_height = bounds.size.height - padding * 2.0;

        for sel_bounds in &prepaint.selections {
            window.paint_quad(fill(*sel_bounds, self.theme.selection));
        }

        for (i, line) in prepaint.lines.iter().enumerate() {
            let line_top = line_height * i as f32 - prepaint.scroll_y;
            if line_top + line_height < px(0.0) || line_top > viewport_height {
                continue;
            }
            let y = content_origin.y + line_top;
            let origin = point(content_origin.x, y);
            let _ = line.paint(origin, line_height, window, cx);
        }

        if let Some((cursor_x, cursor_y)) = prepaint.cursor_pos {
            let cursor_bounds = Bounds::new(
                point(content_origin.x + cursor_x, content_origin.y + cursor_y),
                size(px(2.0), line_height),
            );
            window.paint_quad(fill(cursor_bounds, self.theme.foreground));
        }

        let focus_handle = self.entity.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.entity.clone()),
            cx,
        );

        let content_bounds = Bounds::new(
            content_origin,
            size(
                bounds.size.width - padding * 2.0,
                viewport_height,
            ),
        );

        self.entity.update(cx, |editor, _| {
            editor.last_content_bounds = Some(content_bounds);
            editor.scroll_y = prepaint.scroll_y;
            editor.last_layout = Some(WrappedLayout {
                lines: prepaint.lines.clone(),
                line_ranges: prepaint.line_ranges.clone(),
            });
        });
    }
}
