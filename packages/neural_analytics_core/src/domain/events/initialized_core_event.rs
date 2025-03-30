#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct InitializedCoreEvent;

impl presage::Event for InitializedCoreEvent {
    const NAME: &'static str = "initialized-core";
}