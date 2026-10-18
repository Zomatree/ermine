use freya::prelude::*;

pub fn stoat_label() -> Label {
    label()
        .line_height(1.5)
        .text_height(TextHeightBehavior::All)
}

pub fn stoat_paragraph() -> Paragraph {
    paragraph()
        .line_height(1.5)
        .text_height(TextHeightBehavior::All)
}
