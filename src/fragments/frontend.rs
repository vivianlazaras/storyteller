use super::api::*;
use crate::ApiClient;
use crate::auth::Guard;
use crate::errors::LazyError;
use crate::model::*;
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::response::content::RawHtml;
use rocket::{FromForm, Route, State, form::Form, get, post, routes};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

use crate::assets::graphs::{Entity, EntityExt, Renderable, render_children};
use wrappedviz::rgraph::{Edge, Node};
use wrappedviz::style::*;
use wrappedviz::*;

#[derive(Debug, FromForm)]
pub struct EditFragmentForm<'r> {
    id: Uuid,
    images: Option<Vec<TempFile<'r>>>,
    content: String,
    name: String,
    // I have to think on how to handle reparenting vs adding a parent.
    // maybe if this is set to something else assume reparenting?
    parent: Option<Uuid>,
    category: Option<String>,
    // I also need to make sure the backend only adds the tags if they don't already exist
    // and ensure tags are populated in the edit form.
    tags: Vec<String>,
}

impl<'r> EditFragmentForm<'r> {
    pub fn to_builder(&self) -> FragmentBuilder {
            let content = crate::normalize_newlines(&self.content);

            FragmentBuilder {
                id: Some(self.id),
                parent: self.parent.clone(),
                category: self.category.as_deref(),
                name: self.name.as_str(),
                content: self.content.as_str(),
                tags: &self.tags,
            }
    }
}

#[derive(Debug, FromForm)]
pub struct CreateFragmentForm<'r> {
    image: Option<TempFile<'r>>,
    content: String,
    name: String,
    parent: Option<Uuid>,
    category: Option<String>,
    tags: Vec<String>,
}

impl<'r> CreateFragmentForm<'r> {
    pub fn to_builder(&self) -> FragmentBuilder {
        let content = crate::normalize_newlines(&self.content);

        FragmentBuilder {
            id: None,
            parent: self.parent.clone(),
            category: self.category.as_deref(),
            name: self.name.as_str(),
            content: self.content.as_str(),
            tags: &self.tags,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FragmentRender {
    pub id: Uuid,
    pub name: String,
    pub content: String,
    pub images: Option<Vec<Image>>,
    pub created: String,
    pub last_edited: String,
}

impl FragmentRender {
    pub fn build_node(&self) -> Node {
        let mut node = Node::new(self.id.to_string(), self.name.clone());
        node.set_attr(NodeAttr::Shape(NodeShape::Box));
        node.set_attr(CommonAttr::Tooltip("see more info".to_string()));
        node.set_attr(CommonAttr::Class("fragment".to_string()));
        node.set_attr(CommonAttr::URL(format!("/fragments/{}", self.id)));
        node
    }
}

impl Entity for FragmentRender {
    type Error = LazyError;
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn category(&self) -> &str {
        "fragments"
    }
}

#[rocket::async_trait]
impl<G: CompatGraph<Node = Node, Edge = Edge> + Send> Renderable<G> for FragmentRender {
    type Err = crate::errors::LazyError;

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
        let node = self.build_node();
        graph.add_node(node);

        let request = self.request(api, access_token);
        let characters = self.characters(request.clone()).await?;
        let locations = self.locations(request.clone()).await?;
        let fragments = self.fragments(request.clone()).await?;

        render_children(
            &self.id,
            &characters,
            "character",
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

        Ok(())
    }
}

async fn fetch_fragment(guard: Guard, id: Uuid, api: &State<ApiClient>) -> Result<StoryFragment, LazyError> {
    let url = format!("/fragments/{}", id);
    let fragment: StoryFragment = api
        .get_protected(&url, guard.access_token(), None)
        .await?;
    Ok(fragment)
}

#[post("/", data = "<form>")]
async fn create_fragment<'r>(
    guard: Guard,
    form: Form<CreateFragmentForm<'r>>,
    api: &State<ApiClient>,
) -> Redirect {
    let form = form.into_inner();
    let builder = form.to_builder();
    let newfragment: StoryFragment = builder.build(&api, guard.access_token()).await.unwrap();
    let redirect = if let Some(parent) = builder.parent {
        let category = match &builder.category {
            Some(category) => category,
            None => "stories",
        };
        format!("/{}/{}", category, parent)
    } else {
        format!("/fragments/{}", newfragment.id)
    };
    Redirect::to(redirect)
}

#[post("/edit", data = "<form>")]
async fn edit_fragment<'r>(guard: Guard, api: &State<ApiClient>, form: Form<EditFragmentForm<'r>>) -> Redirect {
    let form = form.into_inner();
    let builder = form.to_builder();
    let newfragment: StoryFragment = builder.edit(&api, guard.access_token()).await.unwrap();
    let redirect = if let Some(parent) = builder.parent {
        let category = match &builder.category {
            Some(category) => category,
            None => "stories",
        };
        format!("/{}/{}", category, parent)
    } else {
        format!("/fragments/{}", newfragment.id)
    };
    Redirect::to(redirect)
}

// id and category can be used to generate a redirect, and link automatically
#[get("/create?<fragment>&<id>&<category>")]
async fn create_fragment_html(
    guard: Guard,
    api: &State<ApiClient>,
    id: Option<Uuid>,
    fragment: Option<Uuid>,
    category: Option<String>,
) -> RawHtml<Template> {
    // id (the entity to link with)
    // category (the type of entity)

    let current = if let Some(fragment) = fragment {
        Some(fetch_fragment(guard, fragment, api).await.unwrap())
    }else {
        None
    };

    let selected: Vec<String> = Vec::new();
    let options = api.get_top_tags(10, 0).await.unwrap();
    RawHtml(Template::render(
        "fragments/create",
        context! { title: "create new fragment", selected, options, parent: id, category, current },
    ))
}

#[get("/<id>")]
async fn get_fragment(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    
    let fragment = fetch_fragment(guard, id, api).await.unwrap();

    RawHtml(Template::render(
        "fragments/fragment",
        context! { title: fragment.name.clone(), fragment },
    ))
}

#[get("/")]
async fn list_fragments(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    let fragments: Vec<StoryFragment> = match api
        .get_protected("/fragments/", guard.access_token(), None)
        .await
        .unwrap()
    {
        Some(fragments) => fragments,
        None => Vec::new(),
    };
    RawHtml(Template::render(
        "fragments/index",
        context!( title: "fragments", fragments ),
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![
        get_fragment,
        create_fragment_html,
        create_fragment,
        list_fragments,
        edit_fragment
    ]
}
