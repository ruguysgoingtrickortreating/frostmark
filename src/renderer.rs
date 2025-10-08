use std::ops::Deref;

use iced::{widget, Element};
use markup5ever_rcdom::Node;

use crate::{
    structs::MarkWidget,
    widgets::{codeblock, link},
};

use super::structs::ChildData;

macro_rules! draw_children {
    ($s:expr, $node:expr, $element:expr, $child_data:expr) => {
        $s.render_children($node, $element, $child_data);
    };
}

impl<
        'a,
        M: Clone + 'a,
        T: widget::button::Catalog + widget::text::Catalog + widget::rule::Catalog + 'a,
        R: iced::advanced::text::Renderer + 'a,
    > MarkWidget<'a, M, T, R>
{
    #[must_use]
    pub(crate) fn traverse_node(
        &self,
        node: &Node,
        element: &mut Element<'a, M, T, R>,
        data: ChildData,
    ) -> bool {
        match &node.data {
            markup5ever_rcdom::NodeData::Document => {
                draw_children!(self, node, element, data);
                true
            }
            markup5ever_rcdom::NodeData::Text { contents } => {
                let text = contents.borrow();
                let size = if data.heading_weight > 0 {
                    36 - (data.heading_weight * 4)
                } else {
                    16
                } as u16;

                *element = if data.monospace {
                    codeblock(
                        text.to_string(),
                        size,
                        self.fn_copying_text.as_ref(),
                        self.font_mono,
                    )
                    .into()
                } else {
                    // TODO: Don't do this for pre elements
                    let t = widget::text(clean_whitespace(&text))
                        .shaping(widget::text::Shaping::Advanced)
                        .size(size);

                    if let (true, Some(f)) =
                        (data.bold, self.font_bold.or(self.font_mono).or(self.font))
                    {
                        t.font(f)
                    } else {
                        t
                    }
                    .into()
                };

                false
            }
            markup5ever_rcdom::NodeData::Element {
                name,
                attrs,
                template_contents: _,
                mathml_annotation_xml_integration_point: _,
            } => self.render_html_inner(name, attrs, node, element, data),
            _ => false,
        }
    }

    #[must_use]
    fn render_html_inner(
        &self,
        name: &html5ever::QualName,
        attrs: &std::cell::RefCell<Vec<html5ever::Attribute>>,
        node: &Node,
        element: &mut Element<'a, M, T, R>,
        data: ChildData,
    ) -> bool {
        let name = name.local.to_string();
        let attrs = attrs.borrow();

        match name.as_str() {
            "center" | "kbd" | "span" => {
                draw_children!(self, node, element, data);
                false
            }
            "html" | "body" | "p" | "div" | "pre" => {
                draw_children!(self, node, element, data);
                true
            }
            "details" | "summary" | "h1" => {
                draw_children!(self, node, element, data.heading(1));
                true
            }
            "h2" => {
                draw_children!(self, node, element, data.heading(2));
                true
            }
            "h3" => {
                draw_children!(self, node, element, data.heading(3));
                true
            }
            "h4" => {
                draw_children!(self, node, element, data.heading(4));
                true
            }
            "blockquote" => {
                let mut t = widget::Column::new().into();
                draw_children!(self, node, &mut t, data);
                *element = widget::stack!(
                    widget::row![widget::Space::with_width(10), t],
                    widget::vertical_rule(2)
                )
                .into();
                true
            }
            "b" | "strong" | "em" | "i" => {
                draw_children!(self, node, element, data.bold());
                false
            }
            "a" => {
                self.draw_link(node, element, &attrs, data);
                false
            }
            "head" | "br" => true,
            "img" => {
                self.draw_image(element, &attrs);
                true
            }
            "code" => {
                draw_children!(self, node, element, data.monospace());
                false
            }
            "hr" => {
                *element = widget::horizontal_rule(4.0).into();
                true
            }
            "ul" => {
                let mut data = data.indent();
                data.li_ordered_number = None;
                draw_children!(self, node, element, data);
                true
            }
            "ol" => {
                draw_children!(self, node, element, data.indent().ordered());
                true
            }
            "li" => {
                let bullet = if let Some(num) = data.li_ordered_number {
                    widget::text!("{num}. ")
                } else {
                    widget::text("- ")
                };
                let mut children: Element<M, T, R> = widget::Column::new().into();
                draw_children!(self, node, &mut children, data);
                *element = widget::row![bullet, children].into();
                true
            }
            _ => {
                *element = widget::text!("[HTML todo: {name}]").into();
                true
            }
        }
    }

    fn draw_image(&self, element: &mut Element<'a, M, T, R>, attrs: &[html5ever::Attribute]) {
        if let Some(attr) = attrs.iter().find(|attr| attr.name.local.deref() == "src") {
            let url = attr.value.to_string();

            let size = attrs
                .iter()
                .find(|attr| {
                    let name = attr.name.local.deref();
                    name == "width" || name == "height"
                })
                .and_then(|n| n.value.deref().parse::<f32>().ok());

            if let Some(func) = self.fn_drawing_image.as_ref() {
                *element = func(&url, size);
            }
        } else {
            *element = widget::text("[HTML error: malformed image]]").into();
        }
    }

    fn draw_link(
        &self,
        node: &Node,
        element: &mut Element<'a, M, T, R>,
        attrs: &std::cell::Ref<'_, Vec<html5ever::Attribute>>,
        data: ChildData,
    ) {
        *element = if let Some(attr) = attrs
            .iter()
            .find(|attr| attr.name.local.to_string().as_str() == "href")
        {
            let url = attr.value.to_string();
            let children_empty = { node.children.borrow().is_empty() };

            let mut children: Element<M, T, R> = widget::Column::new().into();
            draw_children!(self, node, &mut children, data);

            if children_empty {
                children = widget::column!(widget::text(url.clone())).into();
            }
            link(children, &url, self.fn_clicking_link.as_ref()).into()
        } else {
            widget::text("[HTML error: malformed link]]").into()
        };
    }

    fn render_children(&self, node: &Node, element: &mut Element<'a, M, T, R>, data: ChildData) {
        let children = node.children.borrow();

        let mut column = widget::Column::new().spacing(5);
        let mut row =
            widget::Row::new().push_maybe(data.indent.then_some(widget::Space::with_width(16)));

        let mut is_newline = false;

        let mut i = 0;
        for item in children.iter() {
            if is_newline || should_force_newline_for(item) {
                column = column.push(row.wrap());
                row = widget::Row::new()
                    .push_maybe(data.indent.then_some(widget::Space::with_width(16)));
            }
            if is_node_useless(item) {
                continue;
            }

            let mut data = data;
            if data.li_ordered_number.is_some() {
                data.li_ordered_number = Some(i + 1);
            }

            let mut element = widget::column!().into();
            is_newline = self.traverse_node(item, &mut element, data);
            row = row.push(element);

            i += 1;
        }
        column = column.push(row.wrap());
        *element = column.into();
    }
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

fn should_force_newline_for(node: &Node) -> bool {
    if let markup5ever_rcdom::NodeData::Element { name, .. } = &node.data {
        let n: &str = &name.local;
        n == "blockquote"
    } else {
        false
    }
}

impl<
        'a,
        M: Clone + 'a,
        T: widget::button::Catalog + widget::text::Catalog + widget::rule::Catalog + 'a,
        R: iced::advanced::text::Renderer + 'a,
    > From<MarkWidget<'a, M, T, R>> for Element<'a, M, T, R>
{
    fn from(value: MarkWidget<'a, M, T, R>) -> Self {
        let node = &value.state.dom.document;
        let mut elem: Element<'a, M, T, R> = widget::Column::new().into();
        _ = value.traverse_node(node, &mut elem, ChildData::default());
        elem
    }
}

fn clean_whitespace(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|n| !n.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}
