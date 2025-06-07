use uuid::Uuid;
use crate::ApiClient;
use crate::model::StoryFragment;
use super::frontend::FragmentRender;

#[derive(Debug, Clone, Serialize)]
pub struct FragmentBuilder<'a> {
    pub parent: Option<Uuid>,
    pub category: Option<&'a str>,
    pub name: &'a str,
    pub content: &'a str,
    pub tags: &'a Vec<String>,
}

impl<'a> FragmentBuilder<'a> {
    pub fn new() {}
    pub async fn build(&self, api: &ApiClient, access_token: &str) -> anyhow::Result<StoryFragment> {
        api.post("/fragments/", access_token, None, &self).await
    }
}

impl StoryFragment {
    pub fn render(self) -> FragmentRender {
        FragmentRender {
            id: self.id,
            name: self.name,
            content: self.content,
            image: None,
            created: crate::epoch_to_human(self.created),
            // I'll handle last edited when I have edit's implemented
            last_edited: String::from("unimplemented")
        }
    }
}