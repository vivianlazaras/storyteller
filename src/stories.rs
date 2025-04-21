use crate::Record;
use crate::loader::Sled;
use comrak::{Options, markdown_to_html};
use rocket::{
    Route, State, get,
    response::{Redirect, content::RawHtml},
    routes,
};
use rocket_dyn_templates::{Template, context};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountBtn {
    pub text: &'static str,
    pub link: &'static str,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Story {
    pub id: Uuid,
    #[serde(flatten)]
    pub record: Record,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryTitle {
    pub id: Uuid,
    pub name: String,
}

impl From<Story> for StoryTitle {
    fn from(story: Story) -> StoryTitle {
        StoryTitle {
            id: story.id,
            name: story.record.name,
        }
    }
}

impl Story {
    pub fn render_markdown(&self) -> Template {
        let html = markdown_to_html(&self.text, &Options::default());
        //context.insert("name", "World");
        let description = match &self.record.description {
            Some(description) => &description,
            None => "no description provided",
        };
        // Render the template with the given context
        Template::render(
            "story",
            context! {
                title: &self.record.name,
                text: html,
                description: description
            },
        )
    }
}

#[get("/<id>")]
async fn get_story(id: Uuid, sled: &State<Sled>) -> RawHtml<Template> {
    let tree = &sled.stories;
    match tree.get(id) {
        Ok(Some(story_data)) => {
            let story: Story = rmp_serde::decode::from_slice(&story_data).unwrap();
            RawHtml(story.render_markdown())
        }
        _ => {
            unimplemented!();
        }
    }
}

#[get("/")]
async fn list_stories(sled: &State<Sled>) -> RawHtml<Template> {
    let tree = &sled.stories;

    let storylist = tree
        .iter()
        .map(|v| {
            let (key, val) = v.unwrap();
            let mut story: Story = rmp_serde::decode::from_slice(&val).unwrap();
            if story.record.description.is_none() {
                story.record.description = Some(String::from("no description provided"));
            }
            story
        })
        .collect::<Vec<Story>>();
    // return a web page with links to the stories available, maybe in the future provide filter
    RawHtml(Template::render(
        "stories",
        context!( title: "list of stories/writings", stories: storylist ),
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_story, list_stories]
}
