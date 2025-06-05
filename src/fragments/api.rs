use uuid::Uuid;
use crate::ApiClient;
use crate::model::StoryFragment;

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