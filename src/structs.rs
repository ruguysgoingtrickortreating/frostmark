use std::{
    collections::{HashMap, VecDeque},
    ops::Add,
    sync::{Arc, Mutex},
};

use bitflags::bitflags;
use html5ever::{parse_document, tendril::TendrilSink, ParseOpts};
use iced::{
    widget::{self, text_editor::Action},
    Element, Font,
};
use markup5ever_rcdom::RcDom;

#[derive(Debug, Default, Clone, Copy)]
pub struct ChildData {
    pub heading_weight: u16,
    pub flags: ChildDataFlags,

    pub li_ordered_number: Option<usize>,
}

impl ChildData {
    pub fn heading(mut self, weight: u16) -> Self {
        self.heading_weight = weight;
        self
    }

    pub fn insert(mut self, flags: ChildDataFlags) -> Self {
        self.flags.insert(flags);
        self
    }

    pub fn ordered(mut self) -> Self {
        self.li_ordered_number = Some(1);
        self
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ChildDataFlags: u16 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const UNDERLINE = 1 << 2;
        const STRIKETHROUGH = 1 << 3;
        const INDENT = 1 << 4;
        const KEEP_WHITESPACE = 1 << 5;
        const MONOSPACE = 1 << 6;
    }
}

pub struct MarkState {
    pub(crate) dom: RcDom,
    pub(crate) selection_state: HashMap<String, widget::text_editor::Content>,
    pub(crate) selection_queue: Arc<Mutex<VecDeque<(String, Action)>>>,
}

impl MarkState {
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Will never panic
    pub fn with_html(input: &str) -> Self {
        let dom = parse_document(RcDom::default(), ParseOpts::default())
            .from_utf8()
            .read_from(&mut input.as_bytes())
            // Will not panic as reading from &[u8] cannot fail
            .unwrap();

        let mut selection_state = HashMap::new();
        find_codeblocks(&dom.document, &mut selection_state, false);

        Self {
            dom,
            selection_state,
            selection_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    #[must_use]
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
                parse: comrak::ParseOptions::default(),
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

    pub fn update(&mut self) {
        let Some(mut actions) = self.selection_queue.lock().ok() else {
            return;
        };
        for (code, action) in actions.drain(..) {
            if let Some(n) = self.selection_state.get_mut(&code) {
                n.perform(action);
            }
        }
    }
}

fn find_codeblocks(
    dom: &markup5ever_rcdom::Node,
    storage: &mut HashMap<String, widget::text_editor::Content>,
    scan_text: bool,
) {
    let borrow = dom.children.borrow();
    match &dom.data {
        markup5ever_rcdom::NodeData::Element { name, .. } if &name.local == "code" => {
            for child in &*borrow {
                find_codeblocks(child, storage, true);
            }
        }
        markup5ever_rcdom::NodeData::Text { contents } if scan_text => {
            let contents = contents.borrow().to_string();
            let v = widget::text_editor::Content::with_text(&contents);
            storage.insert(contents.clone(), v);
        }
        _ => {
            for child in &*borrow {
                find_codeblocks(child, storage, scan_text);
            }
        }
    }
}

type FClickLink<M> = Box<dyn Fn(&str) -> M>;
type FDrawImage<'a, M, T> = Box<dyn Fn(&str, Option<f32>) -> Element<'a, M, T>>;
type FUpdate<M> = Arc<dyn Fn() -> M>;

pub struct MarkWidget<'a, M, T> {
    pub(crate) state: &'a MarkState,

    pub(crate) font: Option<Font>,
    pub(crate) font_mono: Font,

    pub(crate) fn_clicking_link: Option<FClickLink<M>>,
    pub(crate) fn_drawing_image: Option<FDrawImage<'a, M, T>>,
    pub(crate) fn_select: Option<FUpdate<M>>,
}

impl<'a, M: 'a, T: 'a> MarkWidget<'a, M, T> {
    #[must_use]
    pub fn new(state: &'a MarkState) -> Self {
        Self {
            state,
            font: None,
            font_mono: Font::MONOSPACE,
            fn_clicking_link: None,
            fn_drawing_image: None,
            fn_select: None,
        }
    }

    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.font = Some(font);
        self
    }

    #[must_use]
    pub fn font_mono(mut self, font: Font) -> Self {
        self.font_mono = font;
        self
    }

    #[must_use]
    pub fn on_clicking_link(mut self, f: impl Fn(&str) -> M + 'static) -> Self {
        self.fn_clicking_link = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_drawing_image(
        mut self,
        f: impl Fn(&str, Option<f32>) -> Element<'a, M, T> + 'static,
    ) -> Self {
        self.fn_drawing_image = Some(Box::new(f));
        self
    }

    #[must_use]
    pub fn on_updating_state(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.fn_select = Some(Arc::new(f));
        self
    }
}

pub enum RenderedSpan<'a, M, T> {
    Spans(Vec<widget::text::Span<'a, M, Font>>),
    Elem(Element<'a, M, T>, Emp),
    None,
}

impl<M, T> std::fmt::Debug for RenderedSpan<'_, M, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderedSpan::Spans(spans) => {
                write!(f, "Rs::Spans ")?;
                f.debug_list()
                    .entries(spans.iter().map(|n| &*n.text))
                    .finish()
            }
            RenderedSpan::Elem(_, emp) => write!(f, "Rs::Elem({emp:?})"),
            RenderedSpan::None => write!(f, "Rs::None"),
        }
    }
}

impl<'a, M, T> RenderedSpan<'a, M, T>
where
    M: Clone + 'static,
    T: widget::text::Catalog + 'a,
{
    pub fn is_empty(&self) -> bool {
        match self {
            RenderedSpan::Spans(spans) => spans.is_empty(),
            RenderedSpan::Elem(_, e) => matches!(e, Emp::Empty),
            RenderedSpan::None => true,
        }
    }

    // btw it supports clone so it's fine if we dont ref
    pub fn render(self) -> Element<'a, M, T> {
        match self {
            RenderedSpan::Spans(spans) => widget::rich_text(spans).into(),
            RenderedSpan::Elem(element, _) => element,
            RenderedSpan::None => widget::Column::new().into(),
        }
    }
}

impl<'a, M, T> Add for RenderedSpan<'a, M, T>
where
    M: Clone + 'static,
    T: widget::text::Catalog + 'a,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use RenderedSpan as Rs;
        match (self, rhs) {
            (Rs::None, rhs) => rhs,
            (lhs, Rs::None) => lhs,

            (Rs::Spans(mut spans1), Rs::Spans(spans2)) => {
                spans1.extend(spans2);
                Rs::Spans(spans1)
            }

            (r @ Rs::Spans(_), Rs::Elem(element, e)) => Rs::Elem(
                widget::row![r.render()]
                    .push_maybe(e.has_something().then_some(element))
                    .spacing(5)
                    .wrap()
                    .into(),
                Emp::NonEmpty,
            ),
            (Rs::Elem(element, e), r @ Rs::Spans(_)) => Rs::Elem(
                widget::Row::new()
                    .push_maybe(e.has_something().then_some(element))
                    .push(r.render())
                    .spacing(5)
                    .wrap()
                    .into(),
                Emp::NonEmpty,
            ),
            (Rs::Elem(e1, em1), Rs::Elem(e2, em2)) => Rs::Elem(
                widget::Row::new()
                    .push_maybe(em1.has_something().then_some(e1))
                    .push_maybe(em2.has_something().then_some(e2))
                    .spacing(5)
                    .wrap()
                    .into(),
                Emp::NonEmpty,
            ),
        }
    }
}

impl<'a, M, T, E> From<E> for RenderedSpan<'a, M, T>
where
    M: Clone,
    T: widget::text::Catalog + 'a,
    E: Into<Element<'a, M, T>>,
{
    fn from(value: E) -> Self {
        Self::Elem(value.into(), Emp::NonEmpty)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Emp {
    #[allow(unused)]
    Empty,
    NonEmpty,
}

impl Emp {
    pub fn is_empty(self) -> bool {
        match self {
            Emp::Empty => true,
            Emp::NonEmpty => false,
        }
    }

    pub fn has_something(self) -> bool {
        !self.is_empty()
    }
}
