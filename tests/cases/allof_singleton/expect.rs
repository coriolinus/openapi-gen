pub struct Id(pub Uuid);

pub struct Thing {
    #[serde(skip_deserializing)]
    id: Option<Id>,
}

pub struct WriteableThing {
    id: Id,
}

#[openapi_gen::reexports::async_trait::async_trait]
pub trait Api {}
