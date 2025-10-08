use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use iced::Element;
use markup5ever_rcdom::RcDom;

#[derive(Debug, Default, Clone, Copy)]
pub struct ChildData {
    pub heading_weight: usize,
    pub indent: bool,
    pub bold: bool,
    pub monospace: bool,

    pub li_ordered_number: Option<usize>,
}

impl ChildData {
    pub fn heading(mut self, weight: usize) -> Self {
        self.heading_weight = weight;
        self
    }

    pub fn indent(mut self) -> Self {
        self.indent = true;
        self
    }

    pub fn ordered(mut self) -> Self {
        self.li_ordered_number = Some(1);
        self
    }

    pub fn monospace(mut self) -> Self {
        self.monospace = true;
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
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
