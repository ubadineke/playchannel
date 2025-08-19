//Game Channel
//Game Session

use std::collections::HashMap;

use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

use crate::game_traits::*;

/// Game state channel that extends Paytube for gaming
pub struct PlayChannel {
    /// Game state manager
    game_manager: GameStateManager,
    /// RPC client for Solana
    rpc_client: RpcClient,
    /// Signers for settlement
    keys: Vec<Keypair>,
}

impl PlayChannel {
    /// Create a new game channel with registered game engines
    pub fn new(keys: Vec<Keypair>, rpc_client: RpcClient) -> Self {
        let registry = GameEngineRegistry::new();
        let game_manager = GameStateManager::new(registry);

        Self {
            game_manager,
            rpc_client,
            keys,
        }
    }

    /// Create a new game channel with a pre-configured registry
    pub fn with_registry(
        keys: Vec<Keypair>,
        rpc_client: RpcClient,
        registry: GameEngineRegistry,
    ) -> Self {
        let game_manager = GameStateManager::new(registry);

        Self {
            game_manager,
            rpc_client,
            keys,
        }
    }

    // register game engine?

    /// Create a new game and return the game instance ID
    pub fn create_game(
        &mut self,
        game_type: &GameTypeId,
        players: &[PlayerId],
        stake_amount: u64,
        wagering_config: Option<WageringConfig>,
    ) -> Result<PlayerId, String> {
        let engine = self
            .game_manager
            .get_registry()
            .get_engine(game_type)
            .ok_or_else(|| format!("Game type '{}' not found", game_type))?;

        let final_wagering_config = if let Some(config) = wagering_config {
            if !engine.supports_wagering() {
                return Err("This game does not support wagering".to_string());
            }
            Some(config)
        } else if engine.supports_wagering() {
            engine.default_wagering_config()
        } else {
            None
        };

        //Create game configuration
        let config = GameConfig {
            game_type_id: game_type.clone(),
            max_players: engine.max_players(),
            min_players: engine.min_players(),
            timeout_seconds: 300, //5 minutes
            stake_amount,         //to be investigated
            custom_config: HashMap::new(),
            wagering_config: final_wagering_config,
        };

        //Create game instance
        let game_instance_id = self.game_manager.create_game(&config, players)?;

        Ok(game_instance_id)
    }

    // commit player stake???

    /// Process a game move
    pub fn process_game_move(&mut self, game_move: &GameMove) -> Result<GameActionResult, String> {
        if let Some(game_state) = self
            .game_manager
            .get_game_state_mut(&game_move.game_instance_id)
        {
            game_state.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        // Process the move
        let result = self.game_manager.process_move(game_move)?;

        // Check if game ended
        if let GameActionResult::GameEnded { winner } = &result {
            //Handle game completion
            self.handle_game_completion(&game_move.game_instance_id, *winner)?;
        }

        Ok(result)
    }

    //settle game to the blockchain

    /// Handle game completion
    fn handle_game_completion(
        &mut self,
        game_instance_id: &PlayerId,
        winner: Option<PlayerId>,
    ) -> Result<(), String> {
        // Log game completion
        println!(
            "Game {} completed with winner: {:?}",
            game_instance_id, winner
        );

        // Additional completion logic could go here
        // - Notify players
        // - Update statistics
        // - Trigger automatic settlement

        Ok(())
    }
}
