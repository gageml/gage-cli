use cliclack::ThemeState;
use console::{Emoji, Style};
use tabled::settings::Color;

struct CliTheme;

// Copied from cliclack Theme
const S_BAR_END: Emoji = Emoji("└", "—");
const S_BAR: Emoji = Emoji("│", "|");

impl cliclack::Theme for CliTheme {
    /// Customize the progress spinner.
    fn spinner_chars(&self) -> String {
        "⠋⠙⠚⠒⠂⠂⠒⠲⠴⠦⠖⠒⠐⠐⠒⠓⠋".to_string()
    }

    /// Customize footer.
    ///
    /// Changes caption used for Cancel state.
    fn format_footer_with_message(&self, state: &ThemeState, message: &str) -> String {
        format!(
            "{}\n", // '\n' vanishes by style applying, thus exclude it from styling
            self.bar_color(state).apply_to(match state {
                ThemeState::Active => format!("{S_BAR_END}  {message}"),
                // The following is changed from default Theme:
                ThemeState::Cancel => format!("{S_BAR}\n{S_BAR_END}  Canceled"),
                ThemeState::Submit => format!("{S_BAR}"),
                ThemeState::Error(err) => format!("{S_BAR_END}  {err}"),
            })
        )
    }

    /// Customize checkbox style.
    ///
    /// Removes strikethrough behavior on cancel state (preserves dim).
    fn checkbox_style(&self, state: &ThemeState, selected: bool, active: bool) -> Style {
        match state {
            ThemeState::Cancel if selected => Style::new().dim(),
            ThemeState::Submit if selected => Style::new().dim(),
            _ if !active => Style::new().dim(),
            _ => Style::new(),
        }
    }

    /// Customize input style.
    ///
    /// Removes strikthrough behavior on cancel state (preserves dim).
    fn input_style(&self, state: &ThemeState) -> Style {
        match state {
            ThemeState::Cancel => Style::new().dim(),
            ThemeState::Submit => Style::new().dim(),
            _ => Style::new(),
        }
    }
}

pub fn init() {
    cliclack::set_theme(CliTheme);
}

pub struct Colors;

impl Colors {
    pub fn dim() -> Color {
        Color::new("\u{1b}[2m", "\u{1b}[22m")
    }
}
