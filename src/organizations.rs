use uuid::Uuid;
pub struct Organization {
    id: Uuid,
    name: String,
    description: Option<String>,
}

pub struct OrganizationRender {}