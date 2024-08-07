use yakui::widgets::Text;
use yakui::{column, text, Color};
use yakui_widgets::cosmic_text::FamilyOwned;

pub fn run() {
    column(|| {
        text(32.0, "Default Font");

        let mut text = Text::new(32.0, "Custom Font");
        text.style.attrs.family_owned = FamilyOwned::Monospace;
        text.style.color = Color::GREEN;
        text.show();
    });
}

fn main() {
    bootstrap::start(run as fn());
}
