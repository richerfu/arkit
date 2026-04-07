use super::super::element::Element;
use super::column::ColumnElement;
use super::column_component;

pub type ContainerElement = ColumnElement;

pub fn container(content: impl Into<Element>) -> ContainerElement {
    column_component().child(content.into())
}
