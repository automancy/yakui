use yakui::style::TextStyle;
use yakui::widgets::{Pad, TextBox};
use yakui::{button, center, column, label, row, use_state};

pub fn run() {
    let text = use_state(|| "".to_owned());
    let align = use_state(|| yakui::style::TextAlignment::Start);

    column(|| {
        label("Align:");

        row(|| {
            if button("Start").clicked {
                align.set(yakui::style::TextAlignment::Start);
            }
            if button("Center").clicked {
                align.set(yakui::style::TextAlignment::Center);
            }
            if button("End").clicked {
                align.set(yakui::style::TextAlignment::End);
            }
        });

        center(|| {
            let my_box = TextBox::new(text.borrow().as_str())
                .padding(Pad::all(50.0))
                .style(TextStyle::label().font_size(60.0))
                .placeholder("placeholder");

            let response = my_box.show().into_inner();
            if let Some(new_text) = response.text {
                text.set(new_text);
            }
            if response.activated {
                println!("{}", text.borrow());
            }
        });
    });
}

fn main() {
    bootstrap::start(run as fn());
}
