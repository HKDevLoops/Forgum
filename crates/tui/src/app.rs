//! ratatui config menu implementation.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use forgum_platform::protocol::SceneConfig;
use forgum_platform::shell::Shell;

/// Per-screen quips (Q8=C): one short cow/animal joke per field index.
const QUIPS: &[&str] = &[
    "Why did the cow cross the terminal? To get to the udder side.",
    "A cow's favorite effect is 'moo-rainbow'.",
    "Chameleons paint; cows just moo in color.",
    "Background cows render while you type. Very committed.",
    "This cow has infinite patience. Duration 0 = forever.",
    "High FPS cows are just excited to see you.",
    "These eyes have seen things. Mostly prompts.",
    "A tongue-out cow is a happy cow. Or confused.",
    "'default_shell' = auto-detect. Let the owl guard the perch.",
    "Auto-render on prompt: the beaver builds the dam for you.",
    "Rainbow mode: because monochrome is a missed steak.",
];

/// A dropdown selection backed by a fixed list of string options.
#[derive(Debug)]
struct Dropdown {
    options: Vec<&'static str>,
    index: usize,
}

impl Dropdown {
    fn new(options: Vec<&'static str>, current: &str) -> Self {
        let index = options.iter().position(|o| *o == current).unwrap_or(0);
        Self { options, index }
    }

    fn current(&self) -> String {
        self.options[self.index].to_string()
    }

    fn cycle(&mut self, forward: bool) {
        if forward {
            self.index = (self.index + 1) % self.options.len();
        } else if self.index == 0 {
            self.index = self.options.len() - 1;
        } else {
            self.index -= 1;
        }
    }
}

/// Field identifiers, in display order.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Cow,
    Effect,
    ColorMode,
    Background,
    AutoRenderOnPrompt,
    Duration,
    Fps,
    Eyes,
    Tongue,
    DefaultShell,
}

impl Field {
    const ALL: [Field; 10] = [
        Field::Cow,
        Field::Effect,
        Field::ColorMode,
        Field::Background,
        Field::AutoRenderOnPrompt,
        Field::Duration,
        Field::Fps,
        Field::Eyes,
        Field::Tongue,
        Field::DefaultShell,
    ];

    fn label(self) -> &'static str {
        match self {
            Field::Cow => "cow",
            Field::Effect => "effect",
            Field::ColorMode => "color_mode",
            Field::Background => "background",
            Field::AutoRenderOnPrompt => "auto_render_on_prompt",
            Field::Duration => "duration",
            Field::Fps => "fps",
            Field::Eyes => "eyes",
            Field::Tongue => "tongue",
            Field::DefaultShell => "default_shell",
        }
    }

    fn quip(self) -> &'static str {
        QUIPS[self as usize % QUIPS.len()]
    }
}

/// What the main loop should do after handling an event.
#[derive(Debug)]
pub enum Action {
    Quit,
    Save,
}

/// The interactive config application state.
#[derive(Debug)]
pub struct ConfigApp {
    config: SceneConfig,
    selected: usize,
    /// When `Some`, the user is editing the focused text/numeric field.
    editing: Option<usize>,
    edit_buffer: String,
    effect: Dropdown,
    color_mode: Dropdown,
    default_shell: Dropdown,
    saved: bool,
    footer: String,
}

impl ConfigApp {
    pub fn new(config: SceneConfig) -> Self {
        let effect = Dropdown::new(vec!["static", "rainbow", "fade"], &config.effect);
        let color_mode = Dropdown::new(vec!["rainbow", "solid", "none"], &config.color_mode);
        let default_shell = Dropdown::new(
            vec!["", "bash", "zsh", "fish", "pwsh", "cmd", "powershell"],
            &config.default_shell,
        );
        Self {
            config,
            selected: 0,
            editing: None,
            edit_buffer: String::new(),
            effect,
            color_mode,
            default_shell,
            saved: false,
            footer: String::new(),
        }
    }

    pub fn config(&self) -> &SceneConfig {
        &self.config
    }

    pub fn mark_saved(&mut self) {
        self.saved = true;
        self.footer = "Saved! Press q to quit.".to_string();
    }

    fn focused(&self) -> Field {
        Field::ALL[self.selected]
    }

    /// Handle a raw terminal event. Returns `Ok(Some(Action))` when the caller
    /// should quit or save, otherwise `Ok(None)`.
    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<Option<Action>> {
        let Event::Key(key) = event else {
            return Ok(None);
        };
        if key.kind != event::KeyEventKind::Press {
            return Ok(None);
        }

        if let Some(edit_idx) = self.editing {
            return self.handle_edit_key(edit_idx, key);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Ok(None)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected + 1 < Field::ALL.len() {
                    self.selected += 1;
                }
                Ok(None)
            }
            KeyCode::Char('s') => Ok(Some(Action::Save)),
            KeyCode::Char('q') => Ok(Some(Action::Quit)),
            KeyCode::Enter => {
                self.enter_edit();
                Ok(None)
            }
            KeyCode::Char(' ') => {
                self.toggle_focused();
                Ok(None)
            }
            KeyCode::Left => {
                self.cycle_focused(false);
                Ok(None)
            }
            KeyCode::Right => {
                self.cycle_focused(true);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn enter_edit(&mut self) {
        match self.focused() {
            Field::Cow => {
                self.edit_buffer = self.config.cow.clone();
                self.editing = Some(self.selected);
            }
            Field::Duration => {
                self.edit_buffer = self.config.duration.to_string();
                self.editing = Some(self.selected);
            }
            Field::Fps => {
                self.edit_buffer = self.config.fps.to_string();
                self.editing = Some(self.selected);
            }
            Field::Eyes => {
                self.edit_buffer = self.config.eyes.clone();
                self.editing = Some(self.selected);
            }
            Field::Tongue => {
                self.edit_buffer = self.config.tongue.clone();
                self.editing = Some(self.selected);
            }
            // bool / dropdown fields edit directly (toggle/cycle), no buffer.
            Field::Effect
            | Field::ColorMode
            | Field::Background
            | Field::AutoRenderOnPrompt
            | Field::DefaultShell => {}
        }
    }

    fn handle_edit_key(
        &mut self,
        _edit_idx: usize,
        key: KeyEvent,
    ) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Enter => {
                self.commit_edit();
                self.editing = None;
                Ok(None)
            }
            KeyCode::Esc => {
                self.editing = None;
                Ok(None)
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
                Ok(None)
            }
            KeyCode::Char(c) => {
                // Ignore Ctrl-combos (e.g. Ctrl-C) so the user can bail out.
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(None);
                }
                self.edit_buffer.push(c);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn commit_edit(&mut self) {
        match self.focused() {
            Field::Cow => {
                let v = self.edit_buffer.trim().to_string();
                self.config.cow = if v.is_empty() {
                    "default".to_string()
                } else {
                    v
                };
            }
            Field::Duration => {
                if let Ok(n) = self.edit_buffer.trim().parse::<u32>() {
                    self.config.duration = n;
                }
            }
            Field::Fps => {
                if let Ok(n) = self.edit_buffer.trim().parse::<u16>() {
                    self.config.fps = n;
                }
            }
            Field::Eyes => {
                self.config.eyes = self.edit_buffer.clone();
            }
            Field::Tongue => {
                self.config.tongue = self.edit_buffer.clone();
            }
            _ => {}
        }
    }

    fn toggle_focused(&mut self) {
        match self.focused() {
            Field::Background => self.config.background = !self.config.background,
            Field::AutoRenderOnPrompt => {
                self.config.auto_render_on_prompt = !self.config.auto_render_on_prompt
            }
            _ => {}
        }
    }

    fn cycle_focused(&mut self, forward: bool) {
        match self.focused() {
            Field::Effect => self.effect.cycle(forward),
            Field::ColorMode => self.color_mode.cycle(forward),
            Field::DefaultShell => {
                self.default_shell.cycle(forward);
                let v = self.default_shell.current();
                if !v.is_empty() {
                    if Shell::parse(&v).is_none() {
                        self.footer = format!("warning: unknown shell '{v}'");
                    } else {
                        self.footer.clear();
                    }
                } else {
                    self.footer.clear();
                }
                self.config.default_shell = v;
            }
            _ => {}
        }
    }

    /// Render the whole UI into the given frame.
    pub fn render(&mut self, f: &mut Frame) {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(size);

        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "Forgum",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" config editor"),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        self.render_list(f, chunks[1]);
        self.render_detail(f, chunks[1]);
        self.render_footer(f, chunks[2]);
        self.render_help(f, chunks[3]);
    }

    fn render_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = Field::ALL
            .iter()
            .map(|field| {
                let style = if *field as usize == self.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(field.label(), style)))
            })
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Fields"))
            .highlight_symbol("> ");
        f.render_widget(list, list_rect(area));
    }

    fn render_detail(&self, f: &mut Frame, area: Rect) {
        let rect = detail_rect(area);
        let field = self.focused();
        let (label, value, editing) = self.field_view(field);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", label));
        let inner = if editing {
            Paragraph::new(Line::from(format!("{}█", value))).block(block)
        } else {
            Paragraph::new(Line::from(value)).block(block)
        };
        f.render_widget(Clear, rect);
        f.render_widget(inner, rect);
    }

    fn field_view(&self, field: Field) -> (&'static str, String, bool) {
        let editing = self.editing == Some(field as usize);
        let value = match field {
            Field::Cow => self.config.cow.clone(),
            Field::Effect => self.effect.current(),
            Field::ColorMode => self.color_mode.current(),
            Field::Background => self.config.background.to_string(),
            Field::AutoRenderOnPrompt => self.config.auto_render_on_prompt.to_string(),
            Field::Duration => self.config.duration.to_string(),
            Field::Fps => self.config.fps.to_string(),
            Field::Eyes => self.config.eyes.clone(),
            Field::Tongue => format!("{:?}", self.config.tongue),
            Field::DefaultShell => {
                let v = self.default_shell.current();
                if v.is_empty() {
                    "(auto-detect)".to_string()
                } else {
                    v
                }
            }
        };
        let shown = if editing {
            self.edit_buffer.clone()
        } else {
            value
        };
        (field.label(), shown, editing)
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let text = if !self.footer.is_empty() {
            self.footer.clone()
        } else if self.editing.is_some() {
            "editing: type, Enter to commit, Esc to cancel".to_string()
        } else {
            self.focused().quip().to_string()
        };
        let p = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::Cyan),
        )))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help = "↑/↓ select · Enter edit · Space toggle · ←/→ cycle · s save · q quit";
        let p = Paragraph::new(Line::from(Span::styled(
            help,
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(p, area);
    }
}

/// Left half of the detail area for the field list.
fn list_rect(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width / 3,
        height: area.height,
    }
}

/// Right portion of the detail area for the focused field editor.
fn detail_rect(area: Rect) -> Rect {
    let w = area.width / 3;
    Rect {
        x: area.x + w,
        y: area.y + 1,
        width: area.width - w - 1,
        height: area.height.saturating_sub(2),
    }
}
