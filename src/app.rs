use std::time::Duration;

use gpui::*;

use std::process::Command;

use crate::actions::{OpenConfig, Save, ToggleTheme};
use crate::config::Config;
use crate::document::Document;
use crate::editor::{EditorEvent, EditorView};
use crate::theme::Theme;

const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(5);

pub struct Jid {
    editor: Entity<EditorView>,
    document: Document,
    theme: Theme,
    config: Config,
}

impl Jid {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = Config::load();
        let theme = Theme::from_mode(config.theme);
        let document = Document::new(config.documents_dir.clone());
        let editor = cx.new(|cx| EditorView::new(cx, theme.clone(), config.focus_mode));

        cx.subscribe(&editor, Self::on_editor_event).detach();

        let entity = cx.entity().downgrade();
        window.spawn(cx, {
            let entity = entity.clone();
            async move |cx: &mut AsyncWindowContext| {
                loop {
                    cx.background_executor().timer(AUTOSAVE_INTERVAL).await;
                    let result = entity.update(cx, |this, cx| {
                        this.save_if_modified(cx);
                    });
                    if result.is_err() {
                        break;
                    }
                }
            }
        }).detach();

        Self {
            editor,
            document,
            theme,
            config,
        }
    }

    pub fn editor(&self) -> &Entity<EditorView> {
        &self.editor
    }

    fn on_editor_event(
        &mut self,
        _editor: Entity<EditorView>,
        event: &EditorEvent,
        _cx: &mut Context<Self>,
    ) {
        match event {
            EditorEvent::Modified => {
                self.document.mark_modified();
            }
            EditorEvent::FocusModeChanged(enabled) => {
                self.config.focus_mode = *enabled;
                self.config.save();
            }
        }
    }

    fn save_if_modified(&mut self, cx: &mut Context<Self>) {
        let is_modified = self.editor.read(cx).is_modified();
        if is_modified {
            self.save(cx);
        }
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        let content = self.editor.read(cx).content().to_string();
        if let Err(e) = self.document.save(&content) {
            eprintln!("Failed to save: {}", e);
            return;
        }
        self.editor.update(cx, |editor, _| {
            editor.mark_saved();
        });
        cx.notify();
    }

    fn manual_save(&mut self, _: &Save, _window: &mut Window, cx: &mut Context<Self>) {
        self.save(cx);
    }

    fn toggle_theme(&mut self, _: &ToggleTheme, _window: &mut Window, cx: &mut Context<Self>) {
        self.theme = self.theme.toggled();
        self.config.theme = self.theme.mode;
        self.config.save();
        self.editor.update(cx, |editor, cx| {
            editor.set_theme(self.theme.clone(), cx);
        });
        cx.notify();
    }

    fn open_config(&mut self, _: &OpenConfig, _window: &mut Window, _cx: &mut Context<Self>) {
        let config_path = Config::config_path();
        if !config_path.exists() {
            self.config.save();
        }
        let _ = Command::new("open").arg("-t").arg(&config_path).spawn();
    }
}

impl Render for Jid {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let modified = self.editor.read(cx).is_modified();
        let filename = self.document.filename();
        let title = if modified {
            format!("{} •", filename)
        } else {
            filename
        };

        div()
            .id("jid")
            .key_context("jid")
            .on_action(cx.listener(Self::toggle_theme))
            .on_action(cx.listener(Self::manual_save))
            .on_action(cx.listener(Self::open_config))
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(self.theme.background)
            .text_color(self.theme.foreground)
            .child(
                div()
                    .pt_2()
                    .pb_4()
                    .text_sm()
                    .text_color(self.theme.muted)
                    .child(title)
            )
            .child(self.editor.clone())
    }
}
