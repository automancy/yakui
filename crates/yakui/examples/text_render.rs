use bootstrap::load_common_fonts;
use yakui::style::TextAlignment;
use yakui::widgets::Text;
use yakui::{column, text};

pub fn run() {
    load_common_fonts();

    column(|| {
        text(16.0, "I like to render اللغة العربية in Rust!

عندما يريد العالم أن \u{202a}يتكلّم \u{202c} ، فهو يتحدّث بلغة يونيكود. تسجّل الآن لحضور المؤتمر الدولي العاشر ليونيكود (Unicode Conference)، الذي سيعقد في 10-12 آذار 1997 بمدينة مَايِنْتْس، ألمانيا. و سيجمع المؤتمر بين خبراء من كافة قطاعات الصناعة على الشبكة العالمية انترنيت ويونيكود، حيث ستتم، على الصعيدين الدولي والمحلي على حد سواء مناقشة سبل استخدام يونكود في النظم القائمة وفيما يخص التطبيقات الحاسوبية، الخطوط، تصميم النصوص والحوسبة متعددة اللغات.");

        text(16.0, "I want more terminals to be able to handle ZWJ sequence emoji characters. For example, the service dog emoji 🐕‍🦺 is actually 3 Unicode characters. Kitty handles this fairly well. All VTE-based terminals, however, show \"🐶🦺\".");

        text(
            16.0,
            "
    《施氏食狮史》
石室诗士施氏，嗜狮，誓食十狮。
氏时时适市视狮。
十时，适十狮适市。
是时，适施氏适市。
氏视是十狮，恃矢势，使是十狮逝世。
氏拾是十狮尸，适石室。
石室湿，氏使侍拭石室。
石室拭，氏始试食是十狮。
食时，始识是十狮尸，实十石狮尸。
试释是事。
",
        );

        Text::new(
            16.0,
            "
《三字經》
人之初，性本善。性相近，習相遠。
苟不教，性乃遷。教之道，貴以專。
昔孟母，擇鄰處。子不學，斷機杼。
竇燕山，有義方。教五子，名俱揚。
養不教，父之過。教不嚴，師之惰。
子不學，非所宜。幼不學，老何為。
玉不琢，不成器。人不學，不知義。
為人子，方少時。親師友，習禮儀。
香九齡，能溫席。孝於親，所當執。
融四歲，能讓梨。弟於長，宜先知。
首孝悌，次見聞。知某數，識某文。
一而十，十而百。百而千，千而萬。
三才者，天地人。三光者，日月星。
",
        )
        .style(|style| style.min_width(320.0).align(TextAlignment::End))
        .show();
    });
}

fn main() {
    bootstrap::start(run as fn());
}
