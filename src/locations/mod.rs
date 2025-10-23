use crate::ApiClient;
use crate::assets::graphs::{Entity, EntityExt, Renderable, render_children};
use crate::assets::images::ImageForm;
use crate::auth::Guard;
use crate::errors::LazyError;
use crate::model::{Location, Tag};
use rocket::fs::TempFile;
use std::collections::HashMap;
use wrappedviz::rgraph::{Edge, Node};
use wrappedviz::style::{CommonAttr, NodeAttr, shape::NodeShape};
use wrappedviz::{CompatGraph, CompatNode};

use crate::assets::images::{ImageBuilder, ImageProcessor};
use crate::model::Image;
use rocket::{
    FromForm, Route, State, delete, form::Form, get, post, put, response::Redirect,
    response::content::RawHtml, routes,
};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRender {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub images: Option<Vec<Image>>,
    pub thumbnail: Option<Image>,
    pub tags: Option<Vec<Tag>>,
    pub created: Option<i64>,
}

#[derive(Debug, FromForm)]
pub struct LocationForm<'r> {
    name: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    images: Option<Vec<TempFile<'r>>>,
    imagetags: Option<Vec<String>>,
}

impl<'r> LocationForm<'r> {
    pub async fn to_builder(&self, processor: &ImageProcessor) -> anyhow::Result<LocationBuilder> {
        Ok(LocationBuilder {
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.as_ref().unwrap_or(&Vec::new()).to_vec(),
            thumbnail: self.into_image_builder(processor).await?,
        })
    }
}

impl<'r> ImageForm<'r> for LocationForm<'r> {
    fn images(&self) -> Option<&Vec<TempFile<'r>>> {
        self.images.as_ref()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tags(&self) -> &[String] {
        self.tags.as_deref().unwrap_or(&[])
    }

    fn category(&self) -> &str {
        "locations"
    }

    fn parent(&self) -> Option<Uuid> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationBuilder {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    thumbnail: Option<ImageBuilder>,
}

#[get("/")]
async fn list_places(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    let locations: Vec<LocationRender> = match api
        .get_protected("/locations/", guard.access_token(), None)
        .await
        .unwrap()
    {
        Some(locations) => locations,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "locations/index",
        context! { title: "settings", locations },
    ))
}

#[get("/<id>")]
async fn get_place(guard: Guard, api: &State<ApiClient>, id: Uuid) -> RawHtml<Template> {
    let url = format!("/locations/{}", id);
    let location: LocationRender = api
        .get_protected(&url, guard.access_token(), None)
        .await
        .unwrap();
    println!("location: {:?}", location);
    RawHtml(Template::render(
        "locations/location",
        context! {title: location.name.clone(), location },
    ))
}

#[get("/create")]
async fn create_place_html(api: &State<ApiClient>) -> RawHtml<Template> {
    let options = api.get_top_tags(10, 0).await.unwrap();
    RawHtml(Template::render(
        "locations/create",
        context! { title: "create a setting", options },
    ))
}

#[post("/", data = "<form>")]
async fn create_place<'r>(
    guard: Guard,
    api: &State<ApiClient>,
    form: Form<LocationForm<'r>>,
    processor: &State<ImageProcessor>,
) -> Redirect {
    let locationform = form.into_inner();
    let location = locationform.to_builder(processor).await.unwrap();
    let loc: Location = api
        .post("/locations/", guard.access_token(), None, &location)
        .await
        .unwrap();
    Redirect::to(format!("/locations/{}", loc.id))
}

#[put("/<id>")]
async fn update_place(id: Uuid) {
    unimplemented!();
}

#[delete("/<id>")]
async fn delete_place(id: Uuid) {
    unimplemented!();
}

impl Entity for LocationRender {
    type Error = LazyError;
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn category(&self) -> &str {
        "locations"
    }
}

impl LocationRender {
    pub fn build_node(&self) -> Node {
        let mut node = Node::new(self.id.to_string(), self.name.clone());
        node.set_attr(NodeAttr::Shape(NodeShape::House));
        if let Some(description) = &self.description {
            node.set_attr(CommonAttr::Tooltip(description.clone()));
        } else {
            node.set_attr(CommonAttr::Tooltip("see more info".to_string()))
        }
        node.set_attr(CommonAttr::URL(format!("/locations/{}", self.id)));
        node.set_attr(CommonAttr::Class("location".to_string()));
        node
    }
}

#[rocket::async_trait]
impl<G: CompatGraph<Node = Node, Edge = Edge> + Send> Renderable<G> for LocationRender {
    type Err = LazyError;

    async fn render(
        &self,
        api: &ApiClient,
        access_token: &str,
        graph: &mut G,
        visited: &mut Vec<Uuid>,
    ) -> Result<(), Self::Err> {
        if visited.contains(&self.id) {
            return Ok(());
        }

        visited.push(self.id);
        graph.add_node(self.build_node());

        let request = self.request(api, access_token);

        let fragments = self.fragments(request.clone()).await?;
        let locations = self.locations(request.clone()).await?;

        render_children(
            &self.id,
            &fragments,
            "fragment",
            api,
            access_token,
            graph,
            visited,
        )
        .await?;
        render_children(
            &self.id,
            &locations,
            "location",
            api,
            access_token,
            graph,
            visited,
        )
        .await?;

        Ok(())
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![
        list_places,
        get_place,
        create_place_html,
        create_place,
        update_place,
        delete_place
    ]
}
