use crate::ExampleState;

pub fn run(state: &mut ExampleState) {
    yakui::center(|| {
        yakui::column(|| {
            yakui::button("Button");
            state.checked = yakui::checkbox(state.checked).checked;
            state.name = yakui::textbox(18.0, state.name.clone()).text.clone();
        });
    });
}
