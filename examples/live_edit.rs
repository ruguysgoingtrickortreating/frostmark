use frostmark::{MarkState, MarkWidget};
use iced::{
    widget::{self, text_editor::Content},
    Element, Font, Length, Task,
};

#[derive(Debug, Clone)]
enum Message {
    Nothing,
    EditedText(widget::text_editor::Action),
    ToggleMarkdown(bool),
}

struct App {
    /// Whether to additionally support markdown
    /// alongside HTML
    markdown: bool,
    state: MarkState,
    editor: Content,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Nothing => {}
            Message::EditedText(a) => {
                let is_edit = a.is_edit();
                self.editor.perform(a);
                if is_edit {
                    self.reparse();
                }
            }
            Message::ToggleMarkdown(t) => {
                self.markdown = t;
                self.reparse();
            }
        }
        Task::none()
    }

    fn reparse(&mut self) {
        self.state = if self.markdown {
            MarkState::with_html_and_markdown(&self.editor.text())
        } else {
            MarkState::with_html(&self.editor.text())
        };
    }

    fn view(&self) -> Element<'_, Message> {
        let toggler = widget::row![
            widget::toggler(self.markdown).on_toggle(Message::ToggleMarkdown),
            "Support Markdown"
        ]
        .spacing(10);

        let editor = widget::text_editor(&self.editor)
            .on_action(Message::EditedText)
            .height(Length::Fill);

        widget::column![
            toggler,
            widget::row![
                editor,
                widget::scrollable(
                    MarkWidget::new(&self.state)
                        // These methods are optional
                        .font_bold(Font {
                            weight: iced::font::Weight::ExtraBold,
                            ..Default::default()
                        })
                        .font_mono(Font::MONOSPACE)
                        .on_copying_text(|_| Message::Nothing)
                        .on_clicking_link(|_| Message::Nothing)
                )
                .width(Length::Fill),
            ]
            .spacing(10)
        ]
        .spacing(10)
        .padding(10)
        .into()
    }
}

fn main() {
    iced::application("Hello World", App::update, App::view)
        .run_with(|| {
            (
                App {
                    markdown: true,
                    editor: Content::with_text(DEFAULT),
                    state: MarkState::with_html_and_markdown(DEFAULT),
                },
                Task::none(),
            )
        })
        .unwrap();
}

const DEFAULT: &str = "Type your <b>HTML</b> or *Markdown* here!";

/*
Here's a cool one you can try:


<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Inline and Block Elements Test</title>
</head>
<body>
    <h1>Testing Block and Inline Elements</h1>
    <p>This is a block-level element (paragraph). It should appear on its own line.</p>
    <div>This is another block-level element (div). It should also appear on a new line.</div>
    <span>This is an inline element (span). It should appear on the same line as the previous text.</span>
    <b>This is an inline element (bold). It should appear on the same line as the previous span.</b>
    <i>This is another inline element (italic). It should also appear on the same line as the previous elements.</i>
    <hr>
    <p>Notice how the block elements (like paragraphs and divs) create line breaks, whereas inline elements (like span, bold, and italic) stay on the same line.</p>
</body>
</html>
*/
