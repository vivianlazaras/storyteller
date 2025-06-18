use crate::fragments::api::FragmentBuilder;
use crate::model::StoryFragment;
use petgraph::graph::DiGraph;
use uuid::Uuid;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimelineRender {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub svg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: i64,
    moments: Vec<Moment>,
}

impl Timeline {
    /// this is a particularly complex render implementation because of the reliance on graphviz
    pub fn render(self) -> TimelineRender {
        let mut graph = DiGraph::new();
        let mut moments = self.moments.into_iter();

        if let Some(first_moment) = moments.next() {
            let mut previous_node = graph.add_node(first_moment.fragment.name);
            for moment in moments {
                let current_node = graph.add_node(moment.fragment.name);
                graph.add_edge(previous_node, current_node, "weight");
                previous_node = current_node;
            }
        }

        let svg = if graph.node_count() > 0 {
            Some(crate::assets::graphs::render_graph(graph))
        } else {
            None
        };

        TimelineRender {
            id: self.id,
            name: self.name,
            description: self.description,
            image: None,
            svg,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum TimelineGenerator {
    Fragments(Vec<Uuid>),
    Story(Uuid),
}

#[derive(Debug, Clone, Serialize)]
pub struct TimelineBuilder {
    name: String,
    description: Option<String>,
    generator: Option<TimelineGenerator>,
}

/// this type represents an individual link within a timeline
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Moment {
    id: Uuid,
    timeline: Uuid,
    fragment: StoryFragment,
    idx: i64,
}
