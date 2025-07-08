use crate::api::ApiClient;
use graphviz::style::{NodeAttr, NodeStyle, CommonAttr, color::Color};
use graphviz::*;
use petgraph::{dot::Config, dot::Dot, graph::DiGraph};
use regex::Regex;
use std::path::PathBuf;
use uuid::Uuid;
use graphviz::style::shape::NodeShape;
use std::collections::HashMap;
use rocket::serde::json::Json;

use rocket::{get, post, FromForm, form::Form, response::content::RawHtml};
use rocket_dyn_templates::{Template, context};

use crate::auth::Guard;
use rocket::{routes, Route};

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

fn render_str(graph_str: &str) -> String {
    let context = Context::new();
    let mut gvGraph = Graph::new(graph_str, &context);
    /*for (idx, id) in idmap.into_iter() {
        let node_id = format!("{}", idx);
        gvGraph.set_attr_on_node(&node_id,CommonAttr::Id(id.to_string())).unwrap();
        gvGraph.set_attr_on_node(&node_id, NodeAttr::Style(NodeStyle::Filled)).unwrap();
        gvGraph.set_attr_on_node(&node_id, NodeAttr::FillColor(Color::CORAL)).unwrap();
    }*/
    gvGraph.set_layout(Layout::Dot);
    let svg_slice = context.render(&gvGraph, OutputFormat::Svg).unwrap();
    let svg = String::from_utf8_lossy(&svg_slice);
    //println!("svg: {}", svg);
    strip_svg_dimensions(&svg.to_string())
}

pub(crate) fn render_graph(graph: DiGraph<String, &'static str>, idmap: HashMap<usize, Uuid>) -> String {
    let graph_str = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    println!("graph: {}", graph_str);
    render_str(&graph_str)
}

/*
impl GraphManager {
    /// check to see if graph is already rendered if not it renders it to svg
    pub async fn render(&self, graph: &Graph) -> String {
        let context = Context::new();

    }
}*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    dot: String,
}

#[derive(Debug, Clone, FromForm)]
pub struct GraphForm {
    name: String,
    description: Option<String>,
    dot: String,
}

#[get("/create")]
async fn create_html(guard: Guard) -> RawHtml<Template> {
    RawHtml(
        Template::render("graphs/create", context!{ title: "create a new graph" })
    )
}

#[post("/", data = "<form>")]
async fn create(guard: Guard, form: Form<GraphForm>) {
    unimplemented!();
}

#[post("/preview", format = "json", data = "<data>")]
async fn preview_graph(guard: Guard, data: Json<GraphData>) -> String {
    let text = data.into_inner().dot;
    let svg = render_str(&text);
    svg
}

pub fn get_routes() -> Vec<Route> {
    routes![create_html, create, preview_graph]
}