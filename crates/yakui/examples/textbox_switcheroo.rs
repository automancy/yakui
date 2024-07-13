use yakui::{button, column, unique_textbox, use_state};

#[derive(Debug, Clone, Copy)]
enum Page {
    A,
    B,
}

pub fn run() {
    let page = use_state(|| Page::A);

    column(|| {
        if button("page a").clicked {
            page.set(Page::A);
        }

        if button("page b").clicked {
            page.set(Page::B);
        }

        #[derive(Debug)]
        struct PageA;

        #[derive(Debug)]
        struct PageB;

        match page.get() {
            Page::A => unique_textbox::<PageA>("a", None),
            Page::B => unique_textbox::<PageB>("b", None),
        };
    });
}

fn main() {
    bootstrap::start(run as fn());
}
