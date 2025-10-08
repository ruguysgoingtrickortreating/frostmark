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

// const DEFAULT: &str = "Type your <b>HTML</b> or *Markdown* here!";

const DEFAULT: &str = r"
<h1>Hello from HTML</h1>
<p>Here's a paragraph. It should appear on its own line.</p>
<div>Here's a div. It should also appear on a new line.</div><br>
<span>Here's a span,</span>
<b> and here's a bold section. </b>and
<b>another bold section!</b>
<hr>

# Hello from Markdown

As Sonic the Hedgehog once said,
> The problem with being faster than light
> is that you live in darkness

Welp, gotta buy groceries:
1. Chilli
2. Pepper
3. Sauce
";
