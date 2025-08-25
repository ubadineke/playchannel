pub mod rock_paper_scissors;
use crate::game_traits::GameEngineRegistry;
pub use rock_paper_scissors::RPSEngine;

// Register builtin games here
pub fn register_builtin_games(registry: &mut GameEngineRegistry) {
    // register game here
    // registry.register_engine(engine);
    registry.register_engine(Box::new(RPSEngine::new()));
}
