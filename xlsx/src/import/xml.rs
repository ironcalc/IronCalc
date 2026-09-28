//! A small XML element tree built from `quick-xml` events.
//!
//! Most parts of an xlsx file are small: the workbook, its relationships,
//! styles, a theme, a table, the worksheet once its cells are taken out. They
//! are read as a tree, which is the natural shape for code that looks up a few
//! named children and attributes. The one big part, `sheetData`, is streamed
//! instead (see `sheet_data.rs`) and never becomes a tree.
//!
//! Only what the importer uses is here: elements, their attributes and text.
//! Comments, processing instructions and the document declaration are dropped
//! while the tree is built. Text is handed over as XML 1.0 requires: entity
//! references resolved, line endings normalized to `\n`, attribute values with
//! their line breaks and tabs turned into spaces.

use std::borrow::Cow;
use std::io::BufRead;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesRef, BytesStart, Event};
use quick_xml::XmlVersion;

use crate::error::XlsxError;

/// The namespace of the `xml:` prefix, which needs no declaration.
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XmlAttribute {
    /// The local name, without the prefix.
    pub(crate) name: String,
    /// The namespace the prefix resolves to. An attribute without a prefix has
    /// none: the default namespace does not apply to attributes.
    pub(crate) namespace: Option<String>,
    pub(crate) value: String,
}

/// An element with its attributes, its text and its child elements.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct XmlNode {
    /// The local name, without the prefix.
    name: String,
    attributes: Vec<XmlAttribute>,
    /// The text directly inside the element, character data and CDATA
    /// sections joined. Empty if there is none.
    text: String,
    /// True if the element holds any character data at all, even empty.
    has_text: bool,
    children: Vec<XmlNode>,
}

impl XmlNode {
    /// Parses a document and returns its root element.
    pub(crate) fn parse<R: BufRead>(reader: R) -> Result<XmlNode, XlsxError> {
        let mut reader = quick_xml::Reader::from_reader(reader);
        let mut buffer = Vec::new();
        let mut builder = XmlTreeBuilder::new();
        loop {
            let event = reader.read_event_into(&mut buffer)?;
            if let Event::Eof = event {
                break;
            }
            builder.push(&event)?;
            buffer.clear();
        }
        builder.finish()
    }

    /// Parses a document held in a string; for tests.
    #[cfg(test)]
    pub(crate) fn parse_str(xml: &str) -> Result<XmlNode, XlsxError> {
        XmlNode::parse(xml.as_bytes())
    }

    /// The local name of the element (`cfRule` for `<x14:cfRule>`).
    pub(crate) fn tag_name(&self) -> &str {
        &self.name
    }

    /// True if the local name is `name`, whatever the namespace.
    pub(crate) fn has_tag_name(&self, name: &str) -> bool {
        self.name == name
    }

    /// The value of the attribute `name` that has no namespace prefix.
    pub(crate) fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.namespace.is_none() && a.name == name)
            .map(|a| a.value.as_str())
    }

    /// The value of the attribute `name` in the namespace `namespace`, such as
    /// `r:id`.
    pub(crate) fn attribute_ns(&self, namespace: &str, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.namespace.as_deref() == Some(namespace) && a.name == name)
            .map(|a| a.value.as_str())
    }

    pub(crate) fn has_attribute(&self, name: &str) -> bool {
        self.attribute(name).is_some()
    }

    /// The character data directly inside the element, or None if there is
    /// none at all (`<t/>` and `<t></t>` have none, `<t> </t>` has a space).
    pub(crate) fn text(&self) -> Option<&str> {
        if self.has_text {
            Some(&self.text)
        } else {
            None
        }
    }

    /// The child elements, in document order.
    pub(crate) fn children(&self) -> impl Iterator<Item = &XmlNode> {
        self.children.iter()
    }

    /// The element itself and every element below it, in document order.
    pub(crate) fn descendants(&self) -> impl Iterator<Item = &XmlNode> {
        Descendants { stack: vec![self] }
    }

    /// The first child element with the local name `name`.
    #[cfg(test)]
    pub(crate) fn child(&self, name: &str) -> Option<&XmlNode> {
        self.children.iter().find(|n| n.has_tag_name(name))
    }
}

struct Descendants<'a> {
    stack: Vec<&'a XmlNode>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a XmlNode;

    fn next(&mut self) -> Option<&'a XmlNode> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter().rev());
        Some(node)
    }
}

/// The text of a reference event: a character reference such as `&#10;`
/// or one of the five predefined entities. Any other entity is an error,
/// since xlsx parts declare none.
pub(crate) fn reference_of(reference: &BytesRef) -> Result<Cow<'static, str>, XlsxError> {
    if let Some(character) = reference.resolve_char_ref()? {
        return Ok(Cow::Owned(character.to_string()));
    }
    quick_xml::escape::resolve_predefined_entity(reference)
        .map(Cow::Borrowed)
        .ok_or_else(|| XlsxError::Xml(format!("Unknown entity: &{};", &**reference)))
}

/// The value of an attribute, normalized as XML 1.0 requires: entities
/// resolved, line breaks and tabs replaced by spaces.
pub(crate) fn attribute_value_of<'a>(attribute: &Attribute<'a>) -> Result<Cow<'a, str>, XlsxError> {
    Ok(attribute.normalized_value(XmlVersion::Implicit1_0)?)
}

/// Builds the tree from the events of a document, or of a part of one: the
/// streamed sheet reader hands it every event outside `sheetData`.
pub(crate) struct XmlTreeBuilder {
    /// The open elements, outermost first.
    stack: Vec<XmlNode>,
    /// Namespace declarations in scope: the depth they were declared at, the
    /// prefix and the namespace.
    namespaces: Vec<(usize, String, String)>,
    root: Option<XmlNode>,
}

impl XmlTreeBuilder {
    pub(crate) fn new() -> Self {
        XmlTreeBuilder {
            stack: Vec::new(),
            namespaces: Vec::new(),
            root: None,
        }
    }

    pub(crate) fn push(&mut self, event: &Event) -> Result<(), XlsxError> {
        match event {
            Event::Start(element) => {
                let node = self.open(element)?;
                self.stack.push(node);
            }
            Event::Empty(element) => {
                let node = self.open(element)?;
                self.close(node);
            }
            Event::End(_) => {
                if let Some(node) = self.stack.pop() {
                    self.close(node);
                }
            }
            Event::Text(text) => {
                if let Some(node) = self.stack.last_mut() {
                    node.text
                        .push_str(&text.xml_content(XmlVersion::Implicit1_0));
                    node.has_text = true;
                }
            }
            Event::CData(data) => {
                if let Some(node) = self.stack.last_mut() {
                    node.text
                        .push_str(&data.xml_content(XmlVersion::Implicit1_0));
                    node.has_text = true;
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(node) = self.stack.last_mut() {
                    node.text.push_str(&reference_of(reference)?);
                    node.has_text = true;
                }
            }
            Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_) => {}
            Event::Eof => {}
        }
        Ok(())
    }

    /// The root element, once every element has been closed.
    pub(crate) fn finish(mut self) -> Result<XmlNode, XlsxError> {
        if !self.stack.is_empty() {
            return Err(XlsxError::Xml("Unclosed XML element".to_string()));
        }
        self.root
            .take()
            .ok_or_else(|| XlsxError::Xml("Corrupt XML structure".to_string()))
    }

    fn open(&mut self, element: &BytesStart) -> Result<XmlNode, XlsxError> {
        let depth = self.stack.len();
        let name = element.local_name().as_ref().to_string();
        // Namespace declarations first: an element's own declarations apply
        // to its other attributes whatever the order they are written in.
        let mut attributes = Vec::new();
        for attribute in element.attributes() {
            let attribute = attribute?;
            let key = attribute.key;
            let prefix = key.prefix();
            let local = key.local_name();
            if prefix.is_none() && local.as_ref() == "xmlns" {
                let namespace = attribute_value_of(&attribute)?.into_owned();
                self.namespaces.push((depth, String::new(), namespace));
            } else if prefix.is_some_and(|p| p.as_ref() == "xmlns") {
                let namespace = attribute_value_of(&attribute)?.into_owned();
                self.namespaces
                    .push((depth, local.as_ref().to_string(), namespace));
            } else {
                attributes.push(attribute);
            }
        }
        let attributes = attributes
            .into_iter()
            .map(|attribute| {
                let key = attribute.key;
                let namespace = match key.prefix() {
                    None => None,
                    Some(prefix) if prefix.as_ref() == "xml" => Some(XML_NAMESPACE.to_string()),
                    Some(prefix) => self
                        .namespaces
                        .iter()
                        .rev()
                        .find(|(_, p, _)| p == prefix.as_ref())
                        .map(|(_, _, namespace)| namespace.clone()),
                };
                Ok(XmlAttribute {
                    name: key.local_name().as_ref().to_string(),
                    namespace,
                    value: attribute_value_of(&attribute)?.into_owned(),
                })
            })
            .collect::<Result<Vec<_>, XlsxError>>()?;
        Ok(XmlNode {
            name,
            attributes,
            text: String::new(),
            has_text: false,
            children: Vec::new(),
        })
    }

    fn close(&mut self, node: XmlNode) {
        let depth = self.stack.len();
        // Declarations made on the element that just closed go out of scope
        while self.namespaces.last().is_some_and(|(d, _, _)| *d >= depth) {
            self.namespaces.pop();
        }
        match self.stack.last_mut() {
            Some(parent) => parent.children.push(node),
            None => {
                if self.root.is_none() {
                    self.root = Some(node);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    fn parse(xml: &str) -> XmlNode {
        XmlNode::parse(xml.as_bytes()).unwrap()
    }

    #[test]
    fn elements_attributes_and_text() {
        let root = parse(
            r#"<?xml version="1.0"?><!-- c --><a x="1" y="a&amp;b"><b>hi</b><b/><c><d>deep</d></c>t</a>"#,
        );
        assert_eq!(root.tag_name(), "a");
        assert_eq!(root.attribute("x"), Some("1"));
        assert_eq!(root.attribute("y"), Some("a&b"));
        assert_eq!(root.attribute("z"), None);
        let children: Vec<&str> = root.children().map(|n| n.tag_name()).collect();
        assert_eq!(children, vec!["b", "b", "c"]);
        let bs: Vec<Option<&str>> = root
            .children()
            .filter(|n| n.has_tag_name("b"))
            .map(|n| n.text())
            .collect();
        assert_eq!(bs, vec![Some("hi"), None]);
        assert_eq!(root.text(), Some("t"));
        let all: Vec<&str> = root.descendants().map(|n| n.tag_name()).collect();
        assert_eq!(all, vec!["a", "b", "b", "c", "d"]);
        assert_eq!(
            root.child("c").unwrap().child("d").unwrap().text(),
            Some("deep")
        );
    }

    #[test]
    fn namespaces_on_names_and_attributes() {
        let root = parse(
            r#"<x:ws xmlns:x="urn:x" xmlns:r="urn:r"><x:link r:id="rId1" id="plain"/></x:ws>"#,
        );
        assert_eq!(root.tag_name(), "ws");
        let link = root.child("link").unwrap();
        assert_eq!(link.attribute("id"), Some("plain"));
        assert_eq!(link.attribute_ns("urn:r", "id"), Some("rId1"));
        assert_eq!(link.attribute_ns("urn:x", "id"), None);
    }

    #[test]
    fn line_endings_and_whitespace() {
        let root = parse("<a v=\"1\r\n2\"><t xml:space=\"preserve\"> </t><u>x\r\ny\rz</u></a>");
        assert_eq!(root.attribute("v"), Some("1 2"));
        assert_eq!(root.child("t").unwrap().text(), Some(" "));
        assert_eq!(root.child("u").unwrap().text(), Some("x\ny\nz"));
    }

    #[test]
    fn cdata_and_entities() {
        let root = parse("<a><![CDATA[<b>&]]>&lt;&#65;</a>");
        assert_eq!(root.text(), Some("<b>&<A"));
    }
}
