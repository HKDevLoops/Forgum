//! ratatui config menu implementation with live 30 FPS animation & dark humor.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use forgum_platform::protocol::{ConfigFormat, SceneConfig};
use forgum_platform::shell::Shell;

/// Dark-humored and funky quips per field.
const QUIPS: &[&str] = &[
    "Select your sacrificial bovine. 109+ ASCII creatures await their existential terminal doom.",
    "Default runs each animal's dark signature ritual. Static is for emotional flatliners.",
    "Format war: JSON (strict & cold), YAML (indentation limbo), TOML (civilized peace). One file rule!",
    "Rainbow: because existential dread looks better in TrueColor. Solid/None for stoics.",
    "Haunt your terminal prompt from beyond the grave (non-blocking daemon overlay).",
    "Spawn a fresh ASCII beast on every Enter key. Your RAM knew what it signed up for.",
    "How many seconds until the void claims it? 0 = eternity or until you press Ctrl+C.",
    "Frame rate of the afterlife. 30 = smooth, 60 = euphoric, 240 = GPU toaster mode.",
    "The windows to a nonexistent soul. Try '$$' for greed, 'XX' for dead, 'oo' for innocent.",
    "Sticking its tongue out at compiler errors. 'U' = mocking, ' ' = suppressed screams.",
    "Pick your poison (pwsh, zsh, fish, bash). The engine doesn't judge, but it remembers.",
];

/// A dropdown selection backed by a fixed list of string options.
#[derive(Debug)]
struct Dropdown {
    options: Vec<&'static str>,
    index: usize,
}

impl Dropdown {
    fn new(options: Vec<&'static str>, current: &str) -> Self {
        let index = options
            .iter()
            .position(|o| o.eq_ignore_ascii_case(current))
            .unwrap_or(0);
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
    ConfigFormat,
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
    const ALL: [Field; 11] = [
        Field::Cow,
        Field::Effect,
        Field::ConfigFormat,
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
            Field::ConfigFormat => "config_format",
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
    format: ConfigFormat,
    selected: usize,
    /// When `Some`, the user is editing the focused text/numeric field.
    editing: Option<usize>,
    edit_buffer: String,
    effect: Dropdown,
    format_dropdown: Dropdown,
    color_mode: Dropdown,
    default_shell: Dropdown,
    saved: bool,
    footer: String,
    animation_time: f32,
}

impl ConfigApp {
    pub fn new(config: SceneConfig, format: ConfigFormat) -> Self {
        let effect = Dropdown::new(
            vec![
                "default",
                "static",
                "breathe",
                "float",
                "walk",
                "particles",
                "pulse",
                "glitch",
                "fly",
                "talk",
                "sway",
                "dissolve",
            ],
            &config.effect,
        );
        let format_dropdown = Dropdown::new(vec!["json", "yaml", "toml"], format.extension());
        let color_mode = Dropdown::new(vec!["rainbow", "solid", "none"], &config.color_mode);
        let default_shell = Dropdown::new(
            vec!["", "bash", "zsh", "fish", "pwsh", "cmd", "powershell"],
            &config.default_shell,
        );
        Self {
            config,
            format,
            selected: 0,
            editing: None,
            edit_buffer: String::new(),
            effect,
            format_dropdown,
            color_mode,
            default_shell,
            saved: false,
            footer: String::new(),
            animation_time: 0.0,
        }
    }

    pub fn config(&self) -> &SceneConfig {
        &self.config
    }

    pub fn config_format(&self) -> ConfigFormat {
        ConfigFormat::from_extension(&self.format_dropdown.current()).unwrap_or(self.format)
    }

    pub fn tick(&mut self, dt: f32) {
        self.animation_time += dt;
    }

    pub fn mark_saved(&mut self) {
        self.saved = true;
        self.footer = format!(
            "Saved in {} format! Pasture updated. Press 'q' to exit.",
            self.config_format().display_name()
        );
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
            KeyCode::Char('q') | KeyCode::Esc => Ok(Some(Action::Quit)),
            KeyCode::Enter => {
                self.enter_edit();
                Ok(None)
            }
            KeyCode::Char(' ') => {
                self.toggle_focused();
                Ok(None)
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.cycle_focused(false);
                Ok(None)
            }
            KeyCode::Right | KeyCode::Char('l') => {
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
            | Field::ConfigFormat
            | Field::ColorMode
            | Field::Background
            | Field::AutoRenderOnPrompt
            | Field::DefaultShell => {
                self.cycle_focused(true);
            }
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
                    self.config.fps = n.clamp(1, 240);
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
            Field::Effect => self.cycle_focused(true),
            Field::ConfigFormat => self.cycle_focused(true),
            Field::ColorMode => self.cycle_focused(true),
            Field::DefaultShell => self.cycle_focused(true),
            _ => {}
        }
    }

    fn cycle_focused(&mut self, forward: bool) {
        match self.focused() {
            Field::Effect => {
                self.effect.cycle(forward);
                self.config.effect = self.effect.current();
            }
            Field::ConfigFormat => {
                self.format_dropdown.cycle(forward);
                self.footer = format!(
                    "Config format switched to {} (will migrate on Save)",
                    self.format_dropdown.current().to_uppercase()
                );
            }
            Field::ColorMode => {
                self.color_mode.cycle(forward);
                self.config.color_mode = self.color_mode.current();
            }
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
                Constraint::Length(3),  // title
                Constraint::Min(8),     // field list + detail
                Constraint::Length(10), // cow animated preview
                Constraint::Length(1),  // footer
            ])
            .split(size);

        let fmt_badge = self.config_format().display_name();
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                " 🐮 FORGUM ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Configuration Terminal Matrix  "),
            Span::styled(
                format!("[{fmt_badge}]"),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " (Dark Humored Pasture)",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        self.render_list(f, chunks[1]);
        self.render_detail(f, chunks[1]);
        self.render_preview(f, chunks[2]);
        self.render_footer(f, chunks[3]);
    }

    fn render_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = Field::ALL
            .iter()
            .map(|field| {
                let is_sel = *field as usize == self.selected;
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(field.label(), style),
                ]))
            })
            .collect();
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Pasture Parameters "),
            )
            .highlight_symbol("▶ ");
        f.render_widget(list, list_rect(area));
    }

    fn render_detail(&self, f: &mut Frame, area: Rect) {
        let rect = detail_rect(area);
        let field = self.focused();
        let (label, value, editing) = self.field_view(field);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Parameter: {} ", label));
        let inner = if editing {
            Paragraph::new(Line::from(vec![
                Span::styled(
                    value,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("█", Style::default().fg(Color::Yellow)),
            ]))
            .block(block)
        } else {
            Paragraph::new(Line::from(vec![Span::styled(
                value,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]))
            .block(block)
        };
        f.render_widget(Clear, rect);
        f.render_widget(inner, rect);
    }

    fn field_view(&self, field: Field) -> (&'static str, String, bool) {
        let editing = self.editing == Some(field as usize);
        let value = match field {
            Field::Cow => self.config.cow.clone(),
            Field::Effect => {
                let cur = self.effect.current();
                if cur == "default" {
                    "default (Signature Animal Ritual)".to_string()
                } else {
                    cur
                }
            }
            Field::ConfigFormat => format!(
                "{} (JSON / YAML / TOML)",
                self.format_dropdown.current().to_uppercase()
            ),
            Field::ColorMode => self.color_mode.current(),
            Field::Background => {
                if self.config.background {
                    "true (Daemon Non-blocking Overlay)".to_string()
                } else {
                    "false (One-shot Synchronous)".to_string()
                }
            }
            Field::AutoRenderOnPrompt => {
                if self.config.auto_render_on_prompt {
                    "true (Spawn beast on every prompt)".to_string()
                } else {
                    "false (Manual invocations only)".to_string()
                }
            }
            Field::Duration => {
                if self.config.duration == 0 {
                    "0 (Infinite / Until Signal)".to_string()
                } else {
                    format!("{} seconds", self.config.duration)
                }
            }
            Field::Fps => format!("{} FPS", self.config.fps),
            Field::Eyes => format!("\"{}\"", self.config.eyes),
            Field::Tongue => format!("\"{}\"", self.config.tongue),
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
        let help = "↑/↓ k/j nav · Enter edit · Space toggle · ←/→ h/l cycle · [S]ave · [Q]uit";
        let text = if !self.footer.is_empty() {
            format!("{}  |  {}", self.footer, help)
        } else if self.editing.is_some() {
            format!("editing: type, Enter to commit, Esc to cancel  |  {}", help)
        } else {
            format!("{}  |  {}", self.focused().quip(), help)
        };
        let p = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::Cyan),
        )));
        f.render_widget(p, area);
    }

    fn render_preview(&self, f: &mut Frame, area: Rect) {
        let cow_art = self.load_cow_art();

        // 30 FPS sinusoidal motion simulation for real-time live preview!
        let t = self.animation_time;
        let breathe_offset = ((t * 3.0).sin() * 0.8) as i32;

        let preview_title = format!(
            " 🐮 Live Holographic Preview: {} ({}) [{}] [{}] ",
            self.config.cow,
            self.effect.current(),
            self.color_mode.current(),
            self.config_format().display_name()
        );

        let block = Block::default().borders(Borders::ALL).title(preview_title);

        let rainbow_colors = [
            Color::Red,
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Magenta,
        ];

        let mut lines: Vec<Line> = Vec::new();
        if breathe_offset > 0 {
            lines.push(Line::from(""));
        }

        for (i, l) in cow_art.lines().enumerate() {
            let color = match self.color_mode.current().as_str() {
                "rainbow" => {
                    let color_idx = (i + (t * 5.0) as usize) % rainbow_colors.len();
                    rainbow_colors[color_idx]
                }
                "solid" => Color::Green,
                _ => Color::White,
            };

            // Add dynamic particle sparkles around the cow if effect is default or particles
            let line_text = if self.effect.current() == "default" && (i == 1 || i == 2) {
                let sparkle = if ((t * 8.0) as usize + i) % 3 == 0 {
                    " ✨"
                } else {
                    "   "
                };
                format!("  {l}{sparkle}")
            } else {
                format!("  {l}")
            };

            lines.push(Line::from(Span::styled(
                line_text,
                Style::default().fg(color),
            )));
        }

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
    }

    fn load_cow_art(&self) -> String {
        let data = match forgum_platform::data_dir() {
            Ok(d) => d,
            Err(_) => return Self::fallback_cow(&self.config.eyes, &self.config.tongue),
        };
        let cow_path = data.join("Cows").join(format!("{}.cow", self.config.cow));
        let raw = match std::fs::read_to_string(&cow_path) {
            Ok(s) => s,
            Err(_) => return Self::fallback_cow(&self.config.eyes, &self.config.tongue),
        };
        Self::expand_cow_template(&raw, &self.config.eyes, &self.config.tongue)
    }

    fn expand_cow_template(template: &str, eyes: &str, tongue: &str) -> String {
        let body = if let Some(start) = template.find("<<EOC;") {
            let bstart = start + "<<EOC;".len();
            if let Some(end) = template[bstart..].find("EOC;") {
                &template[bstart..bstart + end]
            } else {
                &template[bstart..]
            }
        } else {
            template
        };
        let mut out = String::with_capacity(body.len());
        for line in body.lines() {
            out.push_str(&line.replace("$eyes", eyes).replace("$tongue", tongue));
            out.push('\n');
        }
        if out.ends_with('\n') {
            out.pop();
        }
        out
    }

    fn fallback_cow(eyes: &str, tongue: &str) -> String {
        let e1 = eyes.chars().next().unwrap_or('o');
        let e2 = eyes.chars().last().unwrap_or('o');
        let t = tongue.chars().next().unwrap_or(' ');
        format!(
            "        \\   {e1}^__{e2}\n         \\  ({e1}{e2})\\_______\n            (__)\\      ({t})\\/\\\n                ||----w |\n                ||     ||"
        )
    }
}

/// Left half of the detail area for the field list.
fn list_rect(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width / 4,
        height: area.height,
    }
}

/// Right portion of the detail area for the focused field editor.
fn detail_rect(area: Rect) -> Rect {
    let w = area.width / 4;
    Rect {
        x: area.x + w,
        y: area.y + 1,
        width: area.width - w - 1,
        height: area.height.saturating_sub(2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_cow_template_different_templates_produce_different_output() {
        let template_a = "$the_cow = <<EOC;\n        $eyes\n   (oo)\\nEOC;";
        let template_b = "$the_cow = <<EOC;\n        $eyes\n   xx\\nEOC;";
        let art_a = ConfigApp::expand_cow_template(template_a, "oo", " ");
        let art_b = ConfigApp::expand_cow_template(template_b, "xx", " ");
        assert!(!art_a.is_empty(), "cow art A must not be empty");
        assert!(!art_b.is_empty(), "cow art B must not be empty");
        assert_ne!(
            art_a, art_b,
            "different templates must produce different art"
        );
    }
}
