use uuid::Uuid;
use super::frontend::TimelineRender;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: i64,   
}

impl Timeline {
    /// this is a particularly complex render implementation because of the reliance on graphviz
    pub fn render(self) -> TimelineRender {
        unimplemented!();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineBuilder {
    name: String,
    description: Option<String>,
}

/// this type represents an individual link within a timeline
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Moment {
    id: Uuid,
    timeline: Uuid,
    entity: Uuid,
    idx: i64
}