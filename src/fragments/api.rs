use super::frontend::FragmentRender;
use crate::ApiClient;
use crate::errors::ApiError;
use crate::model::StoryFragment;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct FragmentBuilder<'a> {
    pub id: Option<Uuid>,
    pub parent: Option<Uuid>,
    pub category: Option<&'a str>,
    pub name: &'a str,
    pub content: &'a str,
    pub tags: &'a Vec<String>,
}

impl<'a> FragmentBuilder<'a> {
    pub fn new() {}
    pub async fn build(
        &self,
        api: &ApiClient,
        access_token: &str,
    ) -> Result<StoryFragment, ApiError> {
        api.post("/fragments/", access_token, None, &self).await
    }

    pub async fn edit(&self, api: &ApiClient, access_token: &str) -> Result<StoryFragment, ApiError> {
        api.put("/fragments/", access_token, &self).await
    }
}

/*
impl StoryFragment {
    pub fn render(self) -> FragmentRender {
        FragmentRender {
            id: self.id,
            name: self.name,
            content: self.content,
            created: crate::epoch_to_human(self.created),
            // I'll handle last edited when I have edit's implemented
            last_edited: String::from("unimplemented"),
        }
    }
}
*/
