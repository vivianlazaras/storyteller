use crate::api::{ApiClient, ApiRequest, get_access_token};
use crate::assets::graphs::{Entity, EntityExt, Renderable, render_children};
use crate::auth::Guard;
use crate::characters::api::CharacterRender;
use crate::errors::LazyError;
use crate::fragments::frontend::FragmentRender;
use crate::model::Note;
use crate::model::{Character, Story, StoryFragment, Tag};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::{FromForm, FromFormField};
use std::collections::HashMap;
use wrappedviz::rgraph::{Edge, Node};
use wrappedviz::style::*;
use wrappedviz::*;

use rocket::{
    Route, State, delete, get, post, put,
    response::{Redirect, content::RawHtml},
    routes,
};

use crate::locations::LocationRender;

use crate::render::SupportedRender;
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountBtn {
    pub text: &'static str,
    pub link: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryRender {
    pub id: Uuid,
    pub created: i64,
    pub last_edited: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub renderer: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct StoryBuilder {
    title: String,
    description: Option<String>,
    renderer: SupportedRender,
    #[field(name = "tags")]
    pub tags: Vec<String>,
    /// The group in which this story is created in, will default to user's default group
    pub group: Option<Uuid>,
}

#[get("/<id>")]
async fn get_story(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let access_token = guard.access_token();
    let url = format!("/stories/{}", id);
    let id_string = id.to_string();
    let story: StoryRender = api.get_protected(&url, &access_token, None).await.unwrap();

    let mut params = HashMap::new();
    let story_str = story.id.to_string();
    params.insert("parent", story_str.as_str());

    let request = api
        .empty_request()
        .access_token(&guard.access_token())
        .params(params);

    let fragments = story.fragments(request.clone()).await.unwrap();
    let characters = story.characters(request.clone()).await.unwrap();
    let locations = story.locations(request.clone()).await.unwrap();
    let tags = story.tags(request.clone()).await.unwrap();
    let notes = story.notes(request).await.unwrap();
    RawHtml(Template::render(
        "stories/story",
        context! { title: story.name.clone(), notes, story, fragments, characters, tags, locations },
    ))
}

#[get("/")]
async fn list_stories(
    guard: Guard,
    api: &State<ApiClient>,
    cookies: &CookieJar<'_>,
) -> RawHtml<Template> {
    //let access_token = cookies.get("access_token").unwrap();
    let resp: Vec<Story> = api
        .get_protected("/stories", &guard.access_token(), None)
        .await
        .unwrap();
    RawHtml(Template::render(
        "stories/index",
        context! { title: "published stories", stories: resp },
    ))
}

#[get("/create")]
async fn create_story_html(api: &State<ApiClient>) -> RawHtml<Template> {
    let selected: Vec<String> = Vec::new();
    let options = api.get_top_tags(10, 0).await.unwrap();
    RawHtml(Template::render(
        "stories/create",
        context! { title: "create story", selected, options },
    ))
}

#[post("/", data = "<story>")]
async fn create_story(
    guard: Guard,
    //auth: &State<rocket_oidc::AuthState>,
    story: Form<StoryBuilder>,
    api: &State<ApiClient>,
) -> Redirect {
    let story = story.into_inner();
    let result: Story = api
        .post("/stories", &guard.access_token(), None, &story)
        .await
        .unwrap();

    Redirect::to(format!("/stories/{}", result.id))
}

pub struct Edit {
    pub id: Uuid,
    pub date: i64,
    pub comment: Option<String>,
    pub addition: bool,
    pub editor: Uuid,
    pub value: String,
}

pub struct StoryEdit {
    edits: Vec<Edit>,
}

#[put("/<id>")]
async fn edit_story(id: Uuid) {
    unimplemented!();
}

#[delete("/<id>")]
async fn delete_story(id: Uuid, api: &State<ApiClient>) -> Redirect {
    api.delete("/stories/", id).await.unwrap();
    Redirect::to("/stories")
}

/*#[post("/search", data = "<term>")]
async fn search(term: Form<>, api: &State<ApiClient>) -> RawHtml<Template> {

}*/

impl StoryRender {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub async fn notes<'a>(&self, mut request: ApiRequest<'a>) -> Result<Vec<Note>, LazyError> {
        let notes: Vec<Note> = match request.route("/notes").send().await? {
            Some(notes) => notes,
            None => Vec::new(),
        };

        Ok(notes)
    }

    pub async fn tags<'a>(&self, mut request: ApiRequest<'a>) -> Result<Vec<Tag>, LazyError> {
        let tagurl = format!("/tags/{}", self.id);
        let tags: Vec<Tag> = match request.route(&tagurl).send().await? {
            Some(tags) => tags,
            None => Vec::new(),
        };
        Ok(tags)
    }

    pub fn build_node(&self) -> Node {
        let mut node = Node::new(self.id.to_string(), self.name.clone());
        //node.set_attr(NodeAttr::Shape(NodeShape::Egg));
        if let Some(description) = &self.description {
            node.set_attr(CommonAttr::Tooltip(description.clone()));
        } else {
            node.set_attr(CommonAttr::Tooltip("see more info".to_string()))
        }
        node.set_attr(CommonAttr::Class("story".to_string()));
        node.set_attr(CommonAttr::URL(format!("/stories/{}", self.id)));
        node
    }
}

#[rocket::async_trait]
impl<G: CompatGraph<Node = Node, Edge = Edge> + Send> Renderable<G> for StoryRender {
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
        // node has been visited.
        visited.push(self.id);

        graph.set_attr(GraphAttr::Root(self.id.to_string()));
        graph.set_attr(GraphAttr::Margin(1.0));
        let node = self.build_node();
        graph.add_node(node);

        let mut params = HashMap::new();
        let story_str = self.id.to_string();
        params.insert("parent", story_str.as_str());

        let request = api
            .empty_request()
            .access_token(access_token)
            .params(params);

        let fragments: Vec<FragmentRender> = self.fragments(request.clone()).await?;
        let characters = self.characters(request.clone()).await?;
        let locations = self.locations(request.clone()).await?;
        let substories = self.stories(request.clone()).await?;

        render_children(&self.id, &substories, "stories", api, access_token, graph, visited).await?;
        // Recursively render children and add edges
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

impl Entity for StoryRender {
    type Error = LazyError;
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn category(&self) -> &str {
        "stories"
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![
        list_stories,
        create_story,
        create_story_html,
        get_story,
        edit_story,
        delete_story
    ]
}
