use crate::api::ApiClient;
use graphviz::style::{NodeAttr, color::Color};
use graphviz::*;
use petgraph::{dot::Config, dot::Dot, graph::DiGraph};
use regex::Regex;
use std::path::PathBuf;
use uuid::Uuid;

pub struct GraphManager {
    graph_dir: PathBuf,
}

pub fn strip_svg_dimensions(svg: &str) -> String {
    let width_re = Regex::new(r#"(?i)\swidth="[^"]*""#).unwrap();
    let height_re = Regex::new(r#"(?i)\sheight="[^"]*""#).unwrap();

    let svg_no_width = width_re.replace(svg, "");
    let svg_no_height = height_re.replace(&svg_no_width, "");

    svg_no_height.to_string()
}

pub(crate) fn render_graph(graph: DiGraph<String, &'static str>) -> String {
    let graph_str = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    println!("graph: {}", graph_str);
    let context = Context::new();
    let mut gvGraph = Graph::new(graph_str, &context);
    gvGraph.set_layout(Layout::Dot);
    gvGraph
        .set_attr_on_node("0", NodeAttr::FillColor(Color::RGB(0, 0, 255)))
        .unwrap();
    let svg_slice = context.render(&gvGraph, OutputFormat::Svg);
    let svg = String::from_utf8_lossy(&svg_slice);
    strip_svg_dimensions(&svg.to_string())
}

/*
impl GraphManager {
    /// check to see if graph is already rendered if not it renders it to svg
    pub async fn render(&self, graph: &Graph) -> String {
        let context = Context::new();

    }
}*/
