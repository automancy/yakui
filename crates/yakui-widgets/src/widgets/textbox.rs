use std::cell::{Cell, RefCell};
use std::mem;

use cosmic_text::Edit;
use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::input::{KeyCode, MouseButton};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;

use crate::font::Fonts;
use crate::shapes::{self, RoundedRectangle};
use crate::style::TextStyle;
use crate::util::widget;
use crate::{colors, pad, use_state};

use super::{Pad, RenderText};

/**
Text that can be edited.

Responds with [TextBoxResponse].
*/
#[derive(Debug, Clone)]
#[non_exhaustive]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct TextBox {
    pub update_text: Option<String>,

    pub style: TextStyle,
    pub padding: Pad,
    pub fill: Option<Color>,
    pub radius: f32,

    /// Whether or not enter triggers a loss of focus and if shift would be needed to override that
    pub inline_edit: bool,
    pub multiline: bool,

    pub selection_halo_color: Color,
    pub selected_bg_color: Color,
    pub cursor_color: Color,

    /// Drawn when no text has been set
    pub placeholder: String,
}

impl TextBox {
    pub fn new(update_text: Option<String>) -> Self {
        Self {
            update_text,

            style: TextStyle::label(),
            padding: Pad::all(8.0),
            fill: Some(colors::BACKGROUND_3),
            radius: 6.0,

            inline_edit: true,
            multiline: false,

            selection_halo_color: Color::WHITE,
            selected_bg_color: Color::CORNFLOWER_BLUE.adjust(0.4),
            cursor_color: Color::RED,

            placeholder: String::new(),
        }
    }

    pub fn with_text(initial_text: &str, updated_text: Option<&str>) -> TextBox {
        let first_time = use_state(|| true);

        if first_time.get() {
            first_time.set(false);

            TextBox::new(Some(initial_text.into()))
        } else {
            TextBox::new(updated_text.map(Into::into))
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<TextBoxResponse> {
        widget::<TextBoxWidget>(self)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DragState {
    None,
    DragStart,
    Dragging,
}

#[derive(Debug)]
pub struct TextBoxWidget {
    props: TextBox,
    active: bool,
    activated: bool,
    lost_focus: bool,
    drag: DragState,
    cosmic_editor: RefCell<Option<cosmic_text::Editor<'static>>>,
    max_size: Cell<Option<(Option<f32>, Option<f32>)>>,
    text_changed: Cell<bool>,
    scale_factor: Cell<Option<f32>>,
}

pub struct TextBoxResponse {
    pub text: Option<String>,
    /// Whether the user pressed "Enter" in this box, only makes sense in inline
    pub activated: bool,
    /// Whether the box lost focus
    pub lost_focus: bool,
}

impl Widget for TextBoxWidget {
    type Props<'a> = TextBox;
    type Response = TextBoxResponse;

    fn new() -> Self {
        Self {
            props: TextBox::new(None),
            active: false,
            activated: false,
            lost_focus: false,
            drag: DragState::None,
            cosmic_editor: RefCell::new(None),
            max_size: Cell::default(),
            text_changed: Cell::default(),
            scale_factor: Cell::default(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;

        let mut style = self.props.style.clone();
        let mut scroll = None;

        let mut is_empty = false;

        let text = self.cosmic_editor.borrow().as_ref().map(|editor| {
            editor.with_buffer(|buffer| {
                scroll = Some(buffer.scroll());
                is_empty = buffer.lines.iter().all(|v| v.text().is_empty());

                buffer
                    .lines
                    .iter()
                    .map(|v| v.text())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        });

        if is_empty {
            // Dim towards background
            style.color = style
                .color
                .lerp(&self.props.fill.unwrap_or(Color::CLEAR), 0.75);
        }

        let render_text = text.clone();
        pad(self.props.padding, || {
            let render_text = (!is_empty)
                .then_some(render_text)
                .flatten()
                .unwrap_or(self.props.placeholder.clone());

            RenderText::with_style(render_text, style).show_with_scroll(scroll);
        });

        Self::Response {
            text: if self.text_changed.take() {
                text.clone()
            } else {
                None
            },
            activated: mem::take(&mut self.activated),
            lost_focus: mem::take(&mut self.lost_focus),
        }
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let max_width = constraints.max.x.is_finite().then_some(
            (constraints.max.x - self.props.padding.offset().x * 2.0) * ctx.layout.scale_factor(),
        );
        let max_height = constraints.max.y.is_finite().then_some(
            (constraints.max.y - self.props.padding.offset().y * 2.0) * ctx.layout.scale_factor(),
        );
        let max_size = (max_width, max_height);

        let fonts = ctx.dom.get_global_or_init(Fonts::default);

        fonts.with_system(|font_system| {
            if self.cosmic_editor.borrow().is_none() {
                self.cosmic_editor.replace(Some(cosmic_text::Editor::new(
                    cosmic_text::BufferRef::Owned(cosmic_text::Buffer::new(
                        font_system,
                        self.props.style.to_metrics(ctx.layout.scale_factor()),
                    )),
                )));
            }

            if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
                if self.scale_factor.get() != Some(ctx.layout.scale_factor())
                    || self.max_size.get() != Some(max_size)
                {
                    editor.with_buffer_mut(|buffer| {
                        buffer.set_metrics(
                            font_system,
                            self.props.style.to_metrics(ctx.layout.scale_factor()),
                        );

                        buffer.set_size(font_system, max_width, max_height);
                    });

                    self.scale_factor.set(Some(ctx.layout.scale_factor()));
                    self.max_size.replace(Some(max_size));
                }

                if let Some(new_text) = &self.props.update_text {
                    self.text_changed.set(true);

                    editor.with_buffer_mut(|buffer| {
                        buffer.set_text(
                            font_system,
                            new_text,
                            self.props.style.attrs.as_attrs(),
                            cosmic_text::Shaping::Advanced,
                        );
                    });

                    editor.set_cursor(cosmic_text::Cursor::new(0, 0));
                }
            }
        });

        self.default_layout(ctx, constraints)
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let fonts = ctx.dom.get_global_or_init(Fonts::default);
        fonts.with_system(|font_system| {
            if let Some(fill_color) = self.props.fill {
                let mut bg = RoundedRectangle::new(layout_node.rect, self.props.radius);
                bg.color = fill_color;
                bg.add(ctx.paint);
            }

            if let Some(editor) = self.cosmic_editor.borrow_mut().as_mut() {
                editor.shape_as_needed(font_system, false);

                let cursor = editor.cursor();
                let selection = editor.selection_bounds();
                editor.with_buffer_mut(|buffer| {
                    let inv_scale_factor = 1.0 / ctx.layout.scale_factor();

                    if let Some((a, b)) = selection {
                        for ((x, w), (y, h)) in buffer
                            .layout_runs()
                            .flat_map(|layout| {
                                layout
                                    .highlight(a, b)
                                    .zip(Some((layout.line_top, layout.line_height)))
                            })
                            .filter(|((_, w), ..)| *w > 0.1)
                        {
                            let mut bg = PaintRect::new(Rect::from_pos_size(
                                layout_node.rect.pos()
                                    + self.props.padding.offset()
                                    + Vec2::new(x, y) * inv_scale_factor,
                                Vec2::new(w, h) * inv_scale_factor,
                            ));
                            bg.color = self.props.selected_bg_color;
                            bg.add(ctx.paint);
                        }
                    }

                    {
                        if let Some(((x, _), (y, h))) = buffer
                            .layout_runs()
                            .flat_map(|layout| {
                                layout
                                    .highlight(cursor, cursor)
                                    .zip(Some((layout.line_top, layout.line_height)))
                            })
                            .next()
                        {
                            let mut bg = PaintRect::new(Rect::from_pos_size(
                                layout_node.rect.pos()
                                    + self.props.padding.offset()
                                    + Vec2::new(x, y) * inv_scale_factor,
                                Vec2::new(1.5, h) * inv_scale_factor,
                            ));
                            bg.color = self.props.cursor_color;
                            bg.add(ctx.paint);
                        }
                    }
                });
            }
        });

        if self.active {
            shapes::selection_halo(ctx.paint, layout_node.rect, self.props.selection_halo_color);
        }

        self.default_paint(ctx);
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::FOCUSED_KEYBOARD | EventInterest::MOUSE_MOVE
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::FocusChanged(focused) => {
                self.active = *focused;
                if !*focused {
                    self.lost_focus = true;
                    if let Some(editor) = self.cosmic_editor.get_mut() {
                        editor.set_cursor(cosmic_text::Cursor::new(0, 0));
                    }
                }
                EventResponse::Sink
            }

            WidgetEvent::MouseMoved(Some(position)) => {
                if self.drag == DragState::DragStart {
                    self.drag = DragState::Dragging;

                    EventResponse::Sink
                } else if self.drag == DragState::Dragging {
                    if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                        let scale_factor = ctx.layout.scale_factor();
                        let relative_pos =
                            *position - layout.rect.pos() - self.props.padding.offset();
                        let glyph_pos = (relative_pos * scale_factor).round().as_ivec2();

                        let fonts = ctx.dom.get_global_or_init(Fonts::default);
                        fonts.with_system(|font_system| {
                            if let Some(editor) = self.cosmic_editor.get_mut() {
                                editor.action(
                                    font_system,
                                    cosmic_text::Action::Drag {
                                        x: glyph_pos.x,
                                        y: glyph_pos.y,
                                    },
                                );
                            }
                        });
                    }

                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }

            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                inside,
                down,
                position,
                modifiers,
                ..
            } => {
                if !inside {
                    return EventResponse::Sink;
                }

                if let Some(layout) = ctx.layout.get(ctx.dom.current()) {
                    let scale_factor = ctx.layout.scale_factor();
                    let relative_pos = *position - layout.rect.pos() - self.props.padding.offset();
                    let glyph_pos = (relative_pos * scale_factor).round().as_ivec2();

                    let fonts = ctx.dom.get_global_or_init(Fonts::default);
                    fonts.with_system(|font_system| {
                        if *down {
                            if self.drag == DragState::None {
                                self.drag = DragState::DragStart;
                            }

                            if let Some(editor) = self.cosmic_editor.get_mut() {
                                if modifiers.shift() {
                                    // TODO wait for cosmic text for shift clicking selection
                                    // Madeline Sparkles: emulating this with a drag
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Drag {
                                            x: glyph_pos.x,
                                            y: glyph_pos.y,
                                        },
                                    );
                                } else {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Click {
                                            x: glyph_pos.x,
                                            y: glyph_pos.y,
                                        },
                                    );
                                }
                            }
                        } else {
                            self.drag = DragState::None;
                        }
                    });
                }

                ctx.input.set_selection(Some(ctx.dom.current()));

                EventResponse::Sink
            }

            WidgetEvent::KeyChanged {
                key,
                down,
                modifiers,
                ..
            } => {
                let fonts = ctx.dom.get_global_or_init(Fonts::default);
                fonts.with_system(|font_system| {
                    if let Some(editor) = self.cosmic_editor.get_mut() {
                        match key {
                            KeyCode::ArrowLeft => {
                                if *down {
                                    if modifiers.ctrl() {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::LeftWord,
                                            ),
                                        );
                                    } else {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(cosmic_text::Motion::Left),
                                        );
                                    }
                                }
                                EventResponse::Sink
                            }

                            KeyCode::ArrowRight => {
                                if *down {
                                    if modifiers.ctrl() {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(
                                                cosmic_text::Motion::RightWord,
                                            ),
                                        );
                                    } else {
                                        editor.action(
                                            font_system,
                                            cosmic_text::Action::Motion(cosmic_text::Motion::Right),
                                        );
                                    }
                                }
                                EventResponse::Sink
                            }

                            KeyCode::ArrowUp => {
                                if *down {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::Up),
                                    );
                                }
                                EventResponse::Sink
                            }

                            KeyCode::ArrowDown => {
                                if *down {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::Down),
                                    );
                                }
                                EventResponse::Sink
                            }

                            KeyCode::PageUp => {
                                if *down {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::PageUp),
                                    );
                                }
                                EventResponse::Sink
                            }

                            KeyCode::PageDown => {
                                if *down {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::PageDown),
                                    );
                                }
                                EventResponse::Sink
                            }

                            KeyCode::Backspace => {
                                if *down {
                                    editor.action(font_system, cosmic_text::Action::Backspace);
                                    self.text_changed.set(true);
                                }
                                EventResponse::Sink
                            }

                            KeyCode::Delete => {
                                if *down {
                                    editor.action(font_system, cosmic_text::Action::Delete);
                                    self.text_changed.set(true);
                                }
                                EventResponse::Sink
                            }

                            KeyCode::Home => {
                                if *down {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::Home),
                                    );
                                }
                                EventResponse::Sink
                            }

                            KeyCode::End => {
                                if *down {
                                    editor.action(
                                        font_system,
                                        cosmic_text::Action::Motion(cosmic_text::Motion::End),
                                    );
                                }
                                EventResponse::Sink
                            }

                            KeyCode::Enter | KeyCode::NumpadEnter => {
                                if *down {
                                    if self.props.inline_edit {
                                        if self.props.multiline && modifiers.shift() {
                                            editor.action(font_system, cosmic_text::Action::Enter);
                                            self.text_changed.set(true);
                                        } else {
                                            self.activated = true;
                                            ctx.input.set_selection(None);
                                        }
                                    } else {
                                        editor.action(font_system, cosmic_text::Action::Enter);
                                        self.text_changed.set(true);
                                    }
                                }
                                EventResponse::Sink
                            }

                            KeyCode::Escape => {
                                if *down {
                                    editor.action(font_system, cosmic_text::Action::Escape);
                                    if self.props.inline_edit {
                                        ctx.input.set_selection(None);
                                    }
                                }
                                EventResponse::Sink
                            }
                            _ => EventResponse::Sink,
                        }
                    } else {
                        EventResponse::Bubble
                    }
                })
            }
            WidgetEvent::TextInput(c, modifiers) => {
                if c.is_control() {
                    return EventResponse::Bubble;
                }

                let fonts = ctx.dom.get_global_or_init(Fonts::default);
                fonts.with_system(|font_system| {
                    if let Some(editor) = self.cosmic_editor.get_mut() {
                        if modifiers.ctrl() {
                            if c.eq_ignore_ascii_case(&'a') {
                                editor.set_selection(cosmic_text::Selection::Line(editor.cursor()));
                            }
                        } else {
                            editor.action(font_system, cosmic_text::Action::Insert(*c));
                            self.text_changed.set(true);
                        }
                    }
                });

                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
    }
}
