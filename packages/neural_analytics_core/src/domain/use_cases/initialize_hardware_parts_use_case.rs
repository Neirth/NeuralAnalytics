use blackbox_di::injectable;
use presage::{BoxedCommand, CommandHandler, Error, Events};

#[injectable]
pub struct InitializeHardwarePartsUseCase;

#[async_trait::async_trait]
impl<C, E> CommandHandler<C, E> for InitializeHardwarePartsUseCase
    where
        E: From<Error>,
{
    fn command_name(&self) -> &'static str {
        "initialize-hardware-parts"
    }

    #[allow(unused_variables)]
    async fn handle(&self, _context: &mut C, command: BoxedCommand) -> Result<Events, E> {
        // Here you would implement the logic to stop the current mode
        // For example, you might want to send a message to the state machine
        // to stop the current mode and transition to a different state.

        // This is just a placeholder implementation.
        println!("(1) Stopping current mode...");

        Ok(Events::new())
    }
}