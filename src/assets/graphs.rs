use crate::api::ApiClient;
use graphviz::style::{NodeAttr, NodeStyle, CommonAttr, color::Color};
use graphviz::*;
use petgraph::{dot::Config, dot::Dot, graph::DiGraph};
use regex::Regex;
use std::path::PathBuf;
use uuid::Uuid;
use graphviz::style::shape::NodeShape;
use std::collections::HashMap;

pub struct GraphManager {
    graph_dir: PathBuf,
}

pub fn strip_svg_dimensions(svg: &str) -> String {
    let width_re = Regex::new(r#"(?i)\swidth="[^"]*""#).unwrap();
    let height_re = Regex::new(r#"(?i)\sheight="[^"]*""#).unwrap();

    let svg_no_width = width_re.replace(svg, "");
    let svg_no_height = height_re.replace(&svg_no_width, "");

    svg_no_height.to_string().replace("&#45;", "-")
}

pub(crate) fn render_graph(graph: DiGraph<String, &'static str>, idmap: HashMap<usize, Uuid>) -> String {
    let graph_str = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    println!("graph: {}", graph_str);
    let context = Context::new();
    let mut gvGraph = Graph::new(graph_str, &context);
    for (idx, id) in idmap.into_iter() {
        let node_id = format!("{}", idx);
        gvGraph.set_attr_on_node(&node_id,CommonAttr::Id(id.to_string())).unwrap();
        gvGraph.set_attr_on_node(&node_id, NodeAttr::Style(NodeStyle::Filled)).unwrap();
        gvGraph.set_attr_on_node(&node_id, NodeAttr::FillColor(Color::CORAL)).unwrap();
    }
    gvGraph.set_layout(Layout::Dot);
    let svg_slice = context.render(&gvGraph, OutputFormat::Svg);
    let svg = String::from_utf8_lossy(&svg_slice);
    //println!("svg: {}", svg);
    strip_svg_dimensions(&svg.to_string())
}

/*
impl GraphManager {
    /// check to see if graph is already rendered if not it renders it to svg
    pub async fn render(&self, graph: &Graph) -> String {
        let context = Context::new();

    }
}*/
