#[derive(Default)]
pub struct Client {
    username: String,
    uuid: u128,
    entity_id: u32
}

impl Client {
    pub fn run(&self, scope: &rayon::Scope){

    }
}
