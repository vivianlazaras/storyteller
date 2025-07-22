use quick_xml::de::from_str;
use serde::Deserialize;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Deserialize)]
#[serde(rename = "rdf:RDF")]
struct Rdf {
    #[serde(rename = "rdf:Description")]
    description: Description,
}

#[derive(Debug, Deserialize)]
struct Description {
    #[serde(rename = "dc:title")]
    title: Option<LangAlt>,

    #[serde(rename = "dc:creator")]
    creator: Option<Seq>,

    #[serde(rename = "dc:subject")]
    subject: Option<Bag>,

    #[serde(rename = "xmp:Rating")]
    rating: Option<u8>,
}

// Support rdf:Alt for multilingual titles
#[derive(Debug, Deserialize)]
struct LangAlt {
    #[serde(rename = "rdf:Alt")]
    alt: Alt,
}

#[derive(Debug, Deserialize)]
struct Alt {
    #[serde(rename = "rdf:li")]
    values: Vec<String>,
}

// Support rdf:Seq for ordered lists
#[derive(Debug, Deserialize)]
struct Seq {
    #[serde(rename = "rdf:li")]
    values: Vec<String>,
}

// Support rdf:Bag for unordered lists
#[derive(Debug, Deserialize)]
struct Bag {
    #[serde(rename = "rdf:li")]
    values: Vec<String>,
}

fn parse_xmp(xmp_packet: &str) -> Result<Rdf, quick_xml::DeError> {
    from_str::<Rdf>(xmp_packet)
}

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DublinCore {
    #[serde(rename = "dc:title")]
    pub title: Option<LangAlt>,

    #[serde(rename = "dc:creator")]
    pub creator: Option<LangAlt>,

    #[serde(rename = "dc:subject")]
    pub subject: Option<RdfBag>,

    #[serde(rename = "dc:description")]
    pub description: Option<LangAlt>,
}

#[derive(Debug, Deserialize)]
pub struct RdfBag {
    #[serde(rename = "Bag")]
    pub bag: RdfLiList,
}

#[derive(Debug, Deserialize)]
pub struct RdfLiList {
    #[serde(rename = "li")]
    pub items: Vec<String>,
}

impl DublinCore {
    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().and_then(|t| t.first())
    }

    pub fn creator(&self) -> Option<&str> {
        unimplemented!();
    }

    pub fn subjects(&self) -> Vec<String> {
        self.subject
            .as_ref()
            .map(|bag| bag.bag.items.clone())
            .unwrap_or_default()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_ref().and_then(|d| d.first())
    }
}

#[derive(Debug, Deserialize)]
pub struct XmpRdf {
    #[serde(rename = "rdf:Description")]
    pub description: DublinCore,
}

#[derive(Debug, Deserialize)]
pub struct XmpMeta {
    #[serde(rename = "rdf:RDF")]
    pub rdf: XmpRdf,
}

impl XmpMeta {
    pub fn title(&self) -> Option<&str> {
        self.rdf.description.title()
    }

    pub fn creator(&self) -> Option<&str> {
        self.rdf.description.creator()
    }

    pub fn subject(&self) -> Option<&str> {
        self.rdf.description.subject()
    }

    pub fn description(&self) -> Option<&str> {
        self.rdf.description.description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn test_parse_basic_xmp() {
        let sample = r#"
            <x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core">
            <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
                    xmlns:dc="http://purl.org/dc/elements/1.1/">
                <rdf:Description>
                <dc:title>
                    <rdf:Alt>
                    <rdf:li xml:lang="x-default">Example Title</rdf:li>
                    </rdf:Alt>
                </dc:title>
                <dc:creator>
                    <rdf:Alt>
                    <rdf:li>Jane Doe</rdf:li>
                    </rdf:Alt>
                </dc:creator>
                <dc:subject>
                    <rdf:Bag>
                        <rdf:li>nsfw</rdf:li>
                        <rdf:li>photography</rdf:li>
                    </rdf:Bag>
                </dc:subject>
                </rdf:Description>
            </rdf:RDF>
            </x:xmpmeta>
            "#;

        let parsed: XmpMeta = from_str(sample).expect("Failed to parse XMP");

        assert_eq!(
            parsed.rdf.description.title.as_ref().unwrap().items[0],
            "Example Title"
        );

        assert_eq!(
            parsed.rdf.description.creator.as_ref().unwrap().items[0],
            "Jane Doe"
        );
    }
}
