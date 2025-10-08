use iced::{advanced, widget, Element};

pub fn codeblock<'a, F, M, T, R>(
    code: String,
    size: u16,
    m: Option<&F>,
    mono: Option<R::Font>,
) -> widget::Button<'a, M, T, R>
where
    R: advanced::text::Renderer + 'a,
    M: 'a,
    T: 'a + widget::button::Catalog + widget::text::Catalog,
    F: Fn(&str) -> M,
{
    let t = widget::text(code.clone()).size(size);
    widget::button(if let Some(mono) = mono {
        t.font(mono)
    } else {
        t
    })
    .on_press_maybe(m.map(|n| n(&code)))
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
