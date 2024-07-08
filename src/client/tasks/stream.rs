use crate::{
    client::{
        state::{global::GlobalState, local::LocalState},
        tasks::Task,
    },
    protocol::InterfaceOut,
};

pub trait TaskStream {
    fn poll(
        &mut self,
        out: &mut dyn InterfaceOut,
        local: &mut LocalState,
        global: &mut GlobalState,
    ) -> Option<Box<dyn Task>>;
}
