use crate::api::ApiClient;
use crate::api::ApiRequest;
use crate::characters::api::CharacterRender;
use crate::errors::ApiError;
use crate::errors::LazyError;
use crate::fragments::frontend::FragmentRender;
use crate::locations::LocationRender;
use crate::stories::StoryRender;
use petgraph::{dot::Config, dot::Dot, graph::DiGraph};
use regex::Regex;
use rocket::serde::json::Json;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use wrappedviz::cgraph::*;
use wrappedviz::rgraph::{Edge as REdge, RustGraph};
use wrappedviz::style::shape::NodeShape;
use wrappedviz::style::{CommonAttr, GraphAttr, NodeAttr, NodeStyle, color::Color};
use wrappedviz::*;
use wrappedviz::{CompatEdge, CompatGraph};

use rocket::{FromForm, State, form::Form, get, post, response::content::RawHtml};
use rocket_dyn_templates::{Template, context};

use crate::auth::Guard;
use rocket::{Route, routes};

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

fn render_str(graph_str: &str) -> (String, String) {
    let context = Context::new();
    let mut gvGraph = Graph::new(graph_str, &context);
    /*for (idx, id) in idmap.into_iter() {
        let node_id = format!("{}", idx);
        gvGraph.set_attr_on_node(&node_id,CommonAttr::Id(id.to_string())).unwrap();
        gvGraph.set_attr_on_node(&node_id, NodeAttr::Style(NodeStyle::Filled)).unwrap();
        gvGraph.set_attr_on_node(&node_id, NodeAttr::FillColor(Color::CORAL)).unwrap();
    }*/
    gvGraph.set_layout(Layout::Neato).unwrap();
    let dot_back = gvGraph.to_dot().unwrap();
    let svg_slice = context.render(&gvGraph, OutputFormat::Svg).unwrap();
    let svg = String::from_utf8_lossy(&svg_slice);
    //println!("svg: {}", svg);
    (strip_svg_dimensions(&svg.to_string()), dot_back)
}

pub(crate) fn render_graph(
    graph: DiGraph<String, &'static str>,
    idmap: HashMap<usize, Uuid>,
) -> String {
    let graph_str = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    println!("graph: {}", graph_str);
    render_str(&graph_str).0
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
    RawHtml(Template::render(
        "graphs/create",
        context! { title: "create a new graph" },
    ))
}

#[post("/", data = "<form>")]
async fn create(guard: Guard, form: Form<GraphForm>) {
    unimplemented!();
}

#[post("/preview", format = "json", data = "<data>")]
async fn preview_graph(guard: Guard, data: Json<GraphData>) -> String {
    let text = data.into_inner().dot;
    let (svg, _) = render_str(&text);
    svg
}

// Assuming CompatGraph and wrappedviz::rgraph::Node are in scope

/// Core trait for something that can be rendered into a graph.
#[rocket::async_trait]
pub trait Renderable<G: CompatGraph> {
    type Err;

    /// Render this entity and its relationships into the graph.
    async fn render(
        &self,
        api: &ApiClient,
        access_token: &str,
        graph: &mut G,
        visited: &mut Vec<Uuid>,
    ) -> Result<(), Self::Err>;
}

pub trait Entity {
    type Error: Send + Sync + 'static + From<LazyError> + From<ApiError>;
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn category(&self) -> &str;
}

/// Extension trait to fetch related renderable entities with default implementations.
/// You can override these in specific entity impls if needed.
#[rocket::async_trait]
pub trait EntityExt: Entity {
    fn request<'a>(&self, api: &'a ApiClient, access_token: &'a str) -> ApiRequest<'a> {
        let request = api
            .empty_request()
            .access_token(access_token)
            .set_param("parent", self.id().to_string())
            .set_param("category", self.category().to_string());
        request
    }
    /// Fetch related fragments.
    async fn fragments<'a>(
        &self,
        request: ApiRequest<'a>,
    ) -> Result<Vec<FragmentRender>, Self::Error> {
        let fragments = match request.route("/fragments/filter").send().await? {
            Some(fragments) => fragments,
            None => Vec::new(),
        };

        Ok(fragments)
    }

    /// grabs substories, this will work for any entity type currently, though the UI will only
    /// reflect stories being subentities of other stories, not stories being subentities of fragments.
    async fn stories<'a>(&self, request: ApiRequest<'a>) -> Result<Vec<StoryRender>, Self::Error> {
        let stories: Vec<StoryRender> =
            match request.route("/stories/filter").send().await? {
                Some(stories) => stories,
                None => Vec::new(),
            };

        Ok(stories)
    }
    /// Fetch related characters.
    async fn characters<'a>(
        &self,
        request: ApiRequest<'a>,
    ) -> Result<Vec<CharacterRender>, Self::Error> {
        // Default implementation
        let characters: Vec<CharacterRender> =
            match request.route("/characters/filter").send().await? {
                Some(characters) => characters,
                None => Vec::new(),
            };

        Ok(characters)
    }

    /// Fetch related locations.
    async fn locations<'a>(
        &self,
        request: ApiRequest<'a>,
    ) -> Result<Vec<LocationRender>, Self::Error> {
        // Default implementation
        let locations: Vec<LocationRender> = match request.route("/locations/filter").send().await?
        {
            Some(locations) => locations,
            None => Vec::new(),
        };
        Ok(locations)
    }
}

// Blanket impl so anyone implementing Renderable automatically implements RenderableExt with defaults.
#[rocket::async_trait]
impl<T> EntityExt for T
where
    T: Entity + Send,
    T::Error: From<LazyError> + Send + Sync + 'static,
{
    // Uses default methods above; override in specific impls if desired.
}

pub(crate) async fn render_children<T, G>(
    parent_id: &Uuid,
    children: &[T],
    label: &str,
    api: &ApiClient,
    access_token: &str,
    graph: &mut G,
    visited: &mut Vec<Uuid>,
) -> Result<(), LazyError>
where
    T: Renderable<G, Err = LazyError> + Sync + Entity,
    G: CompatGraph<Edge = REdge>,
{
    for child in children {
        let child_id = child.id();
        let name = child.name();
        let child_id_str = child_id.to_string();
        let parent_id_str = parent_id.to_string();

        if !visited.contains(&child_id) {
            child.render(api, access_token, graph, visited).await?;
        }

        // Create an edge from the story to the child
        let edge_id = format!("{}->{}", parent_id, child_id);
        //graph.new_edge(label, &parent_id.to_string(), &child_id_str)?;
        debug_assert_ne!(*parent_id, child_id);
        println!("parent: {}, child: {}", parent_id, child_id);
        let edge = REdge::create(parent_id_str, child_id_str);
        graph.add_edge(edge);
        /*if parent_id_str != child_id_str {
            graph.new_edge(edge_id, label, parent_id_str, child_id_str);
        }*/
    }
    Ok(())
}

// for now just render for stories, I will add routes to handle other entites later.
#[get("/generate/<id>")]
async fn generate(guard: Guard, api: &State<ApiClient>, id: Uuid) -> RawHtml<Template> {
    let mut visited = Vec::new();
    let story_str = format!("/stories/{}", id.to_string());
    let story: StoryRender = api
        .request(&story_str)
        .access_token(guard.access_token())
        .send()
        .await
        .unwrap();
    let mut graph = RustGraph::new(story.name().to_string());
    graph.set_attr(GraphAttr::Splines(true));
    story
        .render(api, &guard.access_token(), &mut graph, &mut visited)
        .await
        .unwrap();

    let dot_data = graph.to_dot();
    println!("dot data: {}", dot_data);
    let (result, back) = render_str(&dot_data);
    //assert_eq!(dot_data, back);
    RawHtml(Template::render(
        "graphs/graph",
        context! { title: story.name(), svg: result },
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![create_html, create, preview_graph, generate]
}
