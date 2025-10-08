use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use iced::Element;
use markup5ever_rcdom::RcDom;

#[derive(Debug, Default, Clone, Copy)]
pub struct ChildData {
    pub heading_weight: usize,
    pub indent: bool,
    pub text: TextConfig,
    pub li_ordered: bool,
}

impl ChildData {
    pub fn with_heading(weight: usize) -> Self {
        Self {
            heading_weight: weight,
            ..Default::default()
        }
    }

    pub fn with_indent() -> Self {
        Self {
            indent: true,
            ..Default::default()
        }
    }

    pub fn with_indent_ordered() -> Self {
        Self {
            indent: true,
            li_ordered: true,
            ..Default::default()
        }
    }

    pub fn monospace() -> Self {
        Self {
            text: TextConfig::Mono,
            ..Default::default()
        }
    }

    pub fn bold() -> Self {
        Self {
            text: TextConfig::Bold,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TextConfig {
    Mono,
    Bold,
    #[default]
    Normal,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ElementProperties {
    pub li_ordered_number: Option<usize>,
}

pub struct MarkState {
    pub(crate) dom: RcDom,
}

impl MarkState {
    pub fn with_html(input: &str) -> Self {
        // If you can't fix the chaos, embrace the chaos.
        let input = input
            .replace("<ul>", "<br><ul>")
            .replace("<ol>", "<br><ol>");

        let dom = parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            // Will not panic as reading from &[u8] cannot fail
            .unwrap();

        Self { dom }
    }

    pub fn with_html_and_markdown(input: &str) -> Self {
        let html = comrak::markdown_to_html(
            input,
            &comrak::Options {
                extension: comrak::ExtensionOptions {
                    strikethrough: true,
                    cjk_friendly_emphasis: true,
                    tasklist: true,
                    superscript: true,
                    subscript: true,
                    underline: true,
                    ..Default::default()
                },
                parse: Default::default(),
                render: comrak::RenderOptions {
                    // Our renderer doesn't have the
                    // vulnerabilities of a browser
                    unsafe_: true,
                    ..Default::default()
                },
            },
        );

        Self::with_html(&html)
    }
}

pub struct MarkWidget<'a, M, T, R: iced::advanced::text::Renderer> {
    pub(crate) state: &'a MarkState,

    pub(crate) font: Option<R::Font>,
    pub(crate) font_bold: Option<R::Font>,
    pub(crate) font_mono: Option<R::Font>,

    pub(crate) fn_clicking_link: Option<Box<dyn Fn(&str) -> M>>,
    pub(crate) fn_drawing_image: Option<Box<dyn Fn(&str, Option<f32>) -> Element<'a, M, T, R>>>,
    pub(crate) fn_copying_text: Option<Box<dyn Fn(&str) -> M>>,
}

impl<'a, M: 'a, T: 'a, R> MarkWidget<'a, M, T, R>
where
    R: iced::advanced::text::Renderer + 'a,
{
    pub fn new(state: &'a MarkState) -> Self {
        Self {
            state,
            font: None,
            font_bold: None,
            font_mono: None,
            fn_clicking_link: None,
            fn_drawing_image: None,
            fn_copying_text: None,
        }
    }

    pub fn font(mut self, font: R::Font) -> Self {
        self.font = Some(font);
        self
    }

    pub fn font_bold(mut self, font: R::Font) -> Self {
        self.font_bold = Some(font);
        self
    }

    pub fn font_mono(mut self, font: R::Font) -> Self {
        self.font_mono = Some(font);
        self
    }

    pub fn on_clicking_link<F: Fn(&str) -> M + 'static>(mut self, f: F) -> Self {
        self.fn_clicking_link = Some(Box::new(f));
        self
    }

    pub fn on_drawing_image<F: Fn(&str, Option<f32>) -> Element<'a, M, T, R> + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.fn_drawing_image = Some(Box::new(f));
        self
    }

    pub fn on_copying_text<F: Fn(&str) -> M + 'static>(mut self, f: F) -> Self {
        self.fn_copying_text = Some(Box::new(f));
        self
    }
}
