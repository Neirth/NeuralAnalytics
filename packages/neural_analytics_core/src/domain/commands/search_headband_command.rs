#[derive(Debug)]
pub struct SearchHeadbandCommand;

impl presage::Command for SearchHeadbandCommand {
    const NAME: &'static str = "search-headband";
}