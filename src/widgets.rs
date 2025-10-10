use iced::{
    advanced,
    widget::{self, text::IntoFragment},
    Element,
};

pub fn codeblock<'a, F, M, T>(
    code: &'_ str,
    size: u16,
    m: Option<&F>,
    mono: iced::Font,
) -> widget::Button<'a, M, T>
where
    M: 'a,
    T: 'a + widget::button::Catalog + widget::text::Catalog,
    F: Fn(&str) -> M,
{
    let t = widget::text(code.to_owned()).size(size);
    widget::button(t.font(mono))
        .on_press_maybe(m.map(|n| n(code)))
        .padding(2)
}

pub fn link<'a, M: 'a, T, R: advanced::Renderer + 'a, F>(
    e: impl Into<Element<'a, M, T, R>>,
    url: &str,
    msg: Option<&F>,
) -> widget::Button<'a, M, T, R>
where
    T: widget::button::Catalog + widget::rule::Catalog + 'a,
    F: Fn(&str) -> M,
{
    widget::button(underline(e))
        .on_press_maybe(msg.map(|n| n(url)))
        .padding(0)
}

pub fn link_text<'a, M: 'a, R: advanced::Renderer + advanced::text::Renderer + 'a, F>(
    e: impl IntoFragment<'a>,
    url: &str,
    msg: Option<&F>,
) -> widget::text::Span<'a, M, R::Font>
where
    F: Fn(&str) -> M,
{
    widget::span(e)
        .link_maybe(msg.map(|n| n(url)))
        .underline(true)
}

pub fn underline<'a, M: 'a, T: widget::rule::Catalog + 'a, R: advanced::Renderer + 'a>(
    e: impl Into<Element<'a, M, T, R>>,
) -> widget::Stack<'a, M, T, R> {
    widget::stack!(
        widget::column![e.into()],
        widget::column![
            widget::vertical_space(),
            widget::horizontal_rule(1),
            widget::Space::with_height(1),
        ]
    )
}
