///!
///! The functions in this crate apply different constraints on the data supplied to them

#[derive(Debug, Error)]
pub enum Error {
    
}

/// Represents types of relationships
/// 
/// Direct(name, reverse)
/// Undirect(weight)
/// # Fields
/// `Directed(name, reverse)`
/// name: the name of the relationship
/// reverse: the name of the relationship when rednered from the other direction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RelationshipType {
    Directed(String, String),
    /// there is no reverse direction, there is only one direction
    Linear(String),
    undirected(String),
}

/// created this crate in order to handle mapping relationships
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    kind: RelationshipType,
    root: Uuid,
    entity: Uuid,
    description: Option<String>,
}

pub fn render_linear(reltype: RelationshipType) {}
pub fn render_tree(reltype: RelationshipType) {}
pub fn render_graph(reltypes: Vec<RelationshipType>) {}
