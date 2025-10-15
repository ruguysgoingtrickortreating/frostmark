use iced::{widget, Element, Font, Padding};
use markup5ever_rcdom::{Node, NodeData};

use crate::{
    structs::{
        ChildAlignment, ChildDataFlags, ImageInfo, MarkWidget, RenderedSpan, UpdateMsg,
        UpdateMsgKind,
    },
    widgets::{link, link_text, underline},
};

use super::structs::ChildData;

impl<
        'a,
        M: Clone + 'static,
        T: widget::button::Catalog
            + widget::text::Catalog
            + widget::rule::Catalog
            + widget::text_editor::Catalog
            + widget::checkbox::Catalog
            + 'a,
    > MarkWidget<'a, M, T>
{
    pub(crate) fn traverse_node(
        &mut self,
        node: &Node,
        element: &mut RenderedSpan<'a, M, T>,
        data: ChildData,
    ) {
        match &node.data {
            markup5ever_rcdom::NodeData::Document => self.render_children(node, element, data),

            markup5ever_rcdom::NodeData::Text { contents } => {
                let text = contents.borrow();
                let weight = data.heading_weight;
                let size = match weight {
                    1 => 32,
                    2 => 24,
                    3 => 20,
                    4 => 18,
                    5 => 14,
                    6 => 12,
                    7 => 10,
                    _ => 16,
                };

                *element = if data.flags.contains(ChildDataFlags::MONOSPACE) {
                    self.codeblock(
                        text.to_string(),
                        size,
                        !data.flags.contains(ChildDataFlags::KEEP_WHITESPACE),
                    )
                } else {
                    let mut t =
                        widget::span(if data.flags.contains(ChildDataFlags::KEEP_WHITESPACE) {
                            text.to_string()
                        } else {
                            clean_whitespace(&text)
                        })
                        .size(size);

                    RenderedSpan::Spans(vec![{
                        t = t.font({
                            let mut f = self.font;
                            if data.flags.contains(ChildDataFlags::BOLD) {
                                f.weight = iced::font::Weight::Bold;
                            }
                            if data.flags.contains(ChildDataFlags::ITALIC) {
                                f.style = iced::font::Style::Italic;
                            }
                            f
                        });
                        if data.flags.contains(ChildDataFlags::STRIKETHROUGH) {
                            t = t.strikethrough(true);
                        }
                        if data.flags.contains(ChildDataFlags::UNDERLINE) {
                            t = t.underline(true);
                        }
                        t
                    }])
                };
            }
            markup5ever_rcdom::NodeData::Element { name, attrs, .. } => {
                self.render_html_inner(name, attrs, node, element, data);
            }
            _ => {}
        }
    }

    fn render_html_inner(
        &mut self,
        name: &html5ever::QualName,
        attrs: &std::cell::RefCell<Vec<html5ever::Attribute>>,
        node: &Node,
        element: &mut RenderedSpan<'a, M, T>,
        mut data: ChildData,
    ) {
        let name = name.local.to_string();
        let attrs = attrs.borrow();

        let block_element = is_block_element(node);
        if block_element {
            alignment_read(&mut data, &attrs);
        }

        match name.as_str() {
            "summary" | "kbd" | "span" | "html" | "body" | "p" | "div" => {
                self.render_children(node, element, data);
            }
            "center" => {
                data.alignment = Some(ChildAlignment::Center);
                self.render_children(node, element, data);
            }
            "pre" => {
                self.render_children(node, element, data.insert(ChildDataFlags::KEEP_WHITESPACE));
            }

            "h1" => self.render_children(node, element, data.heading(1)),
            "h2" => self.render_children(node, element, data.heading(2)),
            "h3" => self.render_children(node, element, data.heading(3)),
            "h4" => self.render_children(node, element, data.heading(4)),
            "h5" => self.render_children(node, element, data.heading(5)),
            "h6" => self.render_children(node, element, data.heading(6)),
            "sub" => self.render_children(node, element, data.heading(7)),

            "blockquote" => {
                let mut t = RenderedSpan::None;
                self.render_children(node, &mut t, data);
                *element = widget::stack!(
                    widget::row![widget::Space::with_width(10), t.render()],
                    widget::vertical_rule(2)
                )
                .into();
            }

            "b" | "strong" => {
                self.render_children(node, element, data.insert(ChildDataFlags::BOLD));
            }
            "em" | "i" => self.render_children(node, element, data.insert(ChildDataFlags::ITALIC)),
            "u" => self.render_children(node, element, data.insert(ChildDataFlags::UNDERLINE)),
            "del" | "s" | "strike" => {
                self.render_children(node, element, data.insert(ChildDataFlags::STRIKETHROUGH));
            }
            "code" => self.render_children(node, element, data.insert(ChildDataFlags::MONOSPACE)),

            "details" => self.draw_details(node, element, data),
            "a" => self.draw_link(node, element, &attrs, data),
            "img" => self.draw_image(element, &attrs),

            "br" => *element = widget::Column::new().into(),
            "hr" => *element = widget::horizontal_rule(4.0).into(),
            "head" | "title" | "meta" => {}

            "input" => {
                *element = match get_attr(&attrs, "type").unwrap_or("text") {
                    "checkbox" => {
                        let checked = attrs.iter().any(|attr| &*attr.name.local == "checked");
                        widget::checkbox("", checked).into()
                    }
                    kind => widget::text!("[HTML todo: <input type=\"{kind}\">]").into(),
                }
            }

            "ul" => {
                data.li_ordered_number = None;
                self.render_children(node, element, data);
            }
            "ol" => self.render_children(node, element, data.ordered()),
            "li" => {
                let bullet = if let Some(num) = data.li_ordered_number {
                    widget::text!("{num}. ")
                } else {
                    widget::text("- ")
                };
                let mut children: RenderedSpan<M, T> = widget::Column::new().into();
                self.render_children(node, &mut children, data);
                *element = widget::row![bullet, children.render()].into();
            }
            _ => {
                *element = RenderedSpan::Spans(vec![widget::span(format!("<{name} (TODO)>"))
                    .font(Font {
                        weight: iced::font::Weight::Bold,
                        ..self.font
                    })]);
            }
        }

        if let (true, Some(align)) = (block_element, data.alignment) {
            let elem = std::mem::take(element);
            let align: iced::Alignment = align.into();
            *element = widget::column![elem.render()]
                .width(iced::Length::Fill)
                .align_x(align)
                .into();
        }
    }

    fn draw_details(&mut self, node: &Node, element: &mut RenderedSpan<'a, M, T>, data: ChildData) {
        let mut regular_children = RenderedSpan::None;
        *element = if let (Some(update), Some(state)) = (
            self.fn_update.clone(),
            self.state
                .dropdown_state
                .get(&self.current_dropdown_id)
                .copied(),
        ) {
            let summary = self.get_summary_elements(node, data);
            self.render_children(
                node,
                &mut regular_children,
                data.insert(ChildDataFlags::SKIP_SUMMARY),
            );

            let umsg = UpdateMsg {
                kind: UpdateMsgKind::DetailsToggle(self.current_dropdown_id, !state),
            };

            let link = if let RenderedSpan::Spans(n) = summary {
                RenderedSpan::Spans(
                    n.into_iter()
                        .map(|n| n.link(update(umsg.clone())).underline(true))
                        .collect(),
                )
                .render()
            } else {
                widget::mouse_area(underline(summary.render()))
                    .on_press(update(umsg))
                    .into()
            };

            widget::stack![
                widget::column![link]
                    .push_maybe(state.then_some(regular_children.render()))
                    .padding(Padding::default().left(20).bottom(5)),
                widget::column![if state {
                    widget::text("V").size(12)
                } else {
                    widget::text(">").size(14)
                }]
                .push_maybe(state.then_some(widget::vertical_rule(1)))
                .spacing(5)
                .padding(Padding::default().left(5).top(if state { 5 } else { 0 })),
            ]
            .into()
        } else {
            self.render_children(node, &mut regular_children, data);
            widget::column![
                widget::horizontal_rule(1),
                regular_children.render(),
                widget::horizontal_rule(1),
            ]
            .padding(10)
            .spacing(10)
            .into()
        };
        self.current_dropdown_id += 1;
    }

    fn get_summary_elements(&mut self, node: &Node, data: ChildData) -> RenderedSpan<'a, M, T> {
        node.children
            .borrow()
            .iter()
            .find_map(|elem| {
                if let NodeData::Element { name, .. } = &elem.data {
                    if &*name.local == "summary" {
                        Some(elem)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .map(|n| {
                let mut s = RenderedSpan::None;
                self.traverse_node(&n, &mut s, data);
                s
            })
            .unwrap_or_default()
    }

    fn draw_image(&self, element: &mut RenderedSpan<'a, M, T>, attrs: &[html5ever::Attribute]) {
        if let Some(attr) = attrs.iter().find(|attr| &*attr.name.local == "src") {
            let url = &*attr.value;

            let width = get_attr_num(attrs, "width");
            let height = get_attr_num(attrs, "height");

            if let Some(func) = self.fn_drawing_image.as_deref() {
                *element = func(ImageInfo { url, width, height }).into();
            }
        } else {
            // Error, malformed image
            *element = RenderedSpan::None;
        }
    }

    fn draw_link(
        &mut self,
        node: &Node,
        element: &mut RenderedSpan<'a, M, T>,
        attrs: &std::cell::Ref<'_, Vec<html5ever::Attribute>>,
        data: ChildData,
    ) {
        *element = if let Some(attr) = attrs
            .iter()
            .find(|attr| attr.name.local.to_string().as_str() == "href")
        {
            let url = attr.value.to_string();
            let children_empty = { node.children.borrow().is_empty() };

            let mut children: RenderedSpan<M, T> = widget::Column::new().into();
            self.render_children(node, &mut children, data);

            let msg = self.fn_clicking_link.as_ref();
            if children_empty {
                RenderedSpan::Spans(vec![link_text(widget::span(url.clone()), url, msg)])
            } else if let RenderedSpan::Spans(n) = children {
                RenderedSpan::Spans(
                    n.into_iter()
                        .map(|n| link_text(n, url.clone(), msg))
                        .collect(),
                )
            } else {
                link(children.render(), &url, msg).into()
            }
        } else {
            let mut children: RenderedSpan<M, T> = widget::Column::new().into();
            self.render_children(node, &mut children, data);

            if let RenderedSpan::Spans(n) = children {
                RenderedSpan::Spans(n.into_iter().map(|n| n.underline(true)).collect())
            } else {
                link(children.render(), "", Some(&Self::e).filter(|_| false)).into()
            }
        };
    }

    fn e(_: String) -> M {
        // This will never run, don't worry
        panic!()
    }

    fn render_children(
        &mut self,
        node: &Node,
        element: &mut RenderedSpan<'a, M, T>,
        data: ChildData,
    ) {
        let children = node.children.borrow();

        let mut column = Vec::new();
        let mut row = RenderedSpan::None;

        let mut skipped_summary = false;

        let mut i = 0;
        for item in children.iter() {
            if is_node_useless(item) {
                continue;
            }
            if let NodeData::Element { name, .. } = &item.data {
                if !skipped_summary
                    && data.flags.contains(ChildDataFlags::SKIP_SUMMARY)
                    && &*name.local == "summary"
                {
                    skipped_summary = true;
                    continue;
                }
            }

            let mut data = data;
            if data.li_ordered_number.is_some() {
                data.li_ordered_number = Some(i + 1);
            }
            let mut element = RenderedSpan::None;
            self.traverse_node(item, &mut element, data);

            if is_block_element(item) {
                if !row.is_empty() {
                    let mut old_row = RenderedSpan::None;
                    std::mem::swap(&mut row, &mut old_row);
                    column.push(old_row);
                }

                column.push(element);
            } else {
                row = row + element;
            }

            i += 1;
        }

        if !row.is_empty() {
            column.push(row);
        }

        let len = column.len();
        let is_empty = column.is_empty() || column.iter().filter(|n| !n.is_empty()).count() == 0;

        *element = if is_empty {
            RenderedSpan::None
        } else if len == 1 {
            column.into_iter().next().unwrap()
        } else {
            widget::column(
                column
                    .into_iter()
                    .filter(|n| !n.is_empty())
                    .map(RenderedSpan::render),
            )
            .spacing(5)
            .into()
        };
    }

    fn codeblock(&self, code: String, size: u16, inline: bool) -> RenderedSpan<'a, M, T> {
        if let (false, Some(state), Some(select)) = (
            inline,
            self.state.selection_state.get(&code),
            self.fn_update.clone(),
        ) {
            widget::text_editor(state)
                .size(size)
                .padding(5)
                .font(self.font_mono)
                .on_action(move |action| {
                    select(UpdateMsg {
                        kind: UpdateMsgKind::TextEditor(code.clone(), action),
                    })
                })
                .into()
        } else {
            RenderedSpan::Spans(vec![widget::span(code).size(size).font(self.font_mono)])
        }
    }
}

fn alignment_read(data: &mut ChildData, attrs: &[html5ever::Attribute]) {
    let Some(align) = get_attr(attrs, "align") else {
        return;
    };

    if let "right" | "center" | "centre" = align {
        data.alignment = Some(if align == "right" {
            ChildAlignment::Right
        } else {
            ChildAlignment::Center
        });
    } else if align == "left" {
        data.alignment = None;
    }
}

fn get_attr_num(attrs: &[html5ever::Attribute], attr_name: &str) -> Option<f32> {
    get_attr(attrs, attr_name).and_then(|n| n.parse::<f32>().ok())
}

fn get_attr<'a>(attrs: &'a [html5ever::Attribute], attr_name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|attr| {
            let name = &*attr.name.local;
            name == attr_name
        })
        .map(|n| &*n.value)
}

fn is_node_useless(node: &Node) -> bool {
    if let markup5ever_rcdom::NodeData::Text { contents } = &node.data {
        let contents = contents.borrow();
        let contents = contents.to_string();
        contents.trim().is_empty()
    } else {
        false
    }
}

fn is_block_element(node: &Node) -> bool {
    let markup5ever_rcdom::NodeData::Element { name, .. } = &node.data else {
        return false;
    };
    let n: &str = &name.local;

    matches!(
        n,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "canvas"
            | "dd"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "noscript"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "tfoot"
            | "ul"
            | "video"
            | "br"
            | "summary" // not really block but acts like it
    )
}

impl<
        'a,
        M: Clone + 'static,
        T: widget::button::Catalog
            + widget::text::Catalog
            + widget::rule::Catalog
            + widget::text_editor::Catalog
            + widget::checkbox::Catalog
            + 'a,
    > From<MarkWidget<'a, M, T>> for Element<'a, M, T>
{
    fn from(mut value: MarkWidget<'a, M, T>) -> Self {
        let node = &value.state.dom.document;
        let mut elem: RenderedSpan<'a, M, T> = widget::Column::new().into();
        value.traverse_node(node, &mut elem, ChildData::default());
        elem.render()
    }
}

fn clean_whitespace(input: &str) -> String {
    let mut s = input.split_whitespace().collect::<Vec<&str>>().join(" ");
    if let Some(last) = input.chars().last() {
        if last.is_whitespace() && last != '\n' {
            s.push(last);
        }
    }
    if let Some(first) = input.chars().next() {
        if first.is_whitespace() && first != '\n' {
            s.insert(0, first);
        }
    }
    s
}
