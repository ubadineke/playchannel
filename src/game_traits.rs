//Game Engine
//Wagering Config
//Wagering Type
//player stake *
//wagering state *
//payout *
//Game Config
//Game Move
//Game Engine Registry
//Game State Manager

use solana_sdk::{pubkey::Pubkey, signature::Signature, stake::config};
use std::collections::HashMap;

/// Unique identifier for a game instance
pub type GameInstanceId = Pubkey;

/// Player identifier
pub type PlayerId = Pubkey;

/// Unique identifier for a game type
pub type GameTypeId = String;

/// Move identifier within a game
pub type MoveId = u64;

pub enum GameActionResult {
    /// Action was successful
    Success,
    /// Action failed with a reason
    Failure(String),
    /// Game has ended with a winner
    GameEnded { winner: Option<PlayerId> },
    /// Action requires additional data
    RequiresData(String),
}

/// A generic game move that can represent any action in any game
#[derive(Debug, Clone)]
pub struct GameMove {
    pub game_instance_id: GameInstanceId,
    pub player_id: PlayerId,
    pub move_id: MoveId,
    pub move_data: Vec<u8>, // Serialized game-specific move data
    pub signature: Signature,
    pub timestamp: u64,
}

/// Types of wagering systems
#[derive(Debug, Clone)]
pub enum WageringType {
    WinnerTakesAll,
    SplitPot {
        winner_percentage: u8,
        runner_up_percentage: u8,
    },
    // Tournament {
    //     payouts: Vec<u8>,
    // },
    // Custom{}
}

/// Wagering state for a game
#[derive(Debug, Clone)]
pub struct WageringState {
    pub config: WageringConfig,
    pub player_stakes: HashMap<PlayerId, PlayerStake>,
    pub total_pot: u64,
    pub stakes_committed: bool,
    pub payouts: Option<Vec<Payout>>,
}

/// Payout information
#[derive(Debug, Clone)]
pub struct Payout {
    pub player_id: PlayerId,
    pub amount: u64,
    pub rank: u8, // 1st, 2nd, 3rd, etc.
    pub percentage: u8,
}

/// Wagering configuration for a game
#[derive(Debug, Clone)]
pub struct WageringConfig {
    pub wagering_type: WageringType,
    pub min_stake: u64,
    pub max_stake: Option<u64>,
    pub equal_stakes: bool,
    // pub custom_params: HashMap<String, String>
}

/// A generic game state that can represent any game's state
#[derive(Debug, Clone)]
pub struct GameState {
    pub game_instance_id: GameInstanceId,
    pub game_type_id: GameTypeId,
    pub players: Vec<PlayerId>,
    pub current_player: Option<PlayerId>,
    pub state_data: Vec<u8>, // Serialized game-specific state
    pub move_history: Vec<GameMove>,
    pub is_finished: bool,
    pub winner: Option<PlayerId>,
    pub created_at: u64,
    pub last_updated: u64,
    /// Wagering state (optional - games can choose to support wagering)
    pub wagering_state: Option<WageringState>,
    pub last_activity: u64,
}

/// Player stake information
#[derive(Debug, Clone)]
pub struct PlayerStake {
    pub player_id: PlayerId,
    pub amount: u64,
    pub token_mint: Option<Pubkey>, // None for SOL, Some for SPL tokens
    pub committed: bool,
    pub committed_at: u64,
}

/// Configuration for a game instance
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub game_type_id: GameTypeId,
    pub max_players: u8,
    pub min_players: u8,
    pub timeout_seconds: u64,
    pub stake_amount: u64,
    pub custom_config: HashMap<String, String>, // Game-specific configuration
    /// Wagering configuration (optional)
    pub wagering_config: Option<WageringConfig>,
}

pub trait GameEngine: Send + Sync {
    //unique identifier
    fn game_type_id(&self) -> GameTypeId;
    //display name
    fn display_name(&self) -> String;
    //description
    fn description(&self) -> String;
    //maximum players
    fn max_players(&self) -> u8;
    //minimum players
    fn min_players(&self) -> u8;
    //support for wagering
    fn supports_wagering(&self) -> bool {
        false
    }
    //default wager config
    fn default_wagering_config(&self) -> Option<WageringConfig> {
        None
    }
    //create game
    fn create_game(&self, config: &GameConfig, players: &[PlayerId]) -> Result<GameState, String>;

    //validate move
    /// Validate if a move is legal in the current game state
    fn validate_move(&self, game_state: &GameState, game_move: &GameMove) -> GameActionResult;

    //apply move
    /// Apply a move to the game state and return the new state
    fn apply_move(&self, game_state: &GameState, game_move: &GameMove)
        -> Result<GameState, String>;

    //game ended/check winner
    /// Check if the game has ended and determine the winner
    fn check_game_end(&self, game_state: &GameState) -> Option<PlayerId>;

    //get current player
    //calculate payouts
    fn calculate_payouts(&self, game_state: &GameState) -> Result<Vec<Payout>, String> {
        if let Some(wagering_state) = &game_state.wagering_state {
            match wagering_state.config.wagering_type {
                WageringType::WinnerTakesAll => {
                    if let Some(winner) = &game_state.winner {
                        Ok(vec![Payout {
                            player_id: *winner,
                            amount: wagering_state.total_pot,
                            rank: 1,
                            percentage: 100,
                        }])
                    } else {
                        Ok(wagering_state
                            .player_stakes
                            .iter()
                            .map(|(player_id, stake)| Payout {
                                player_id: *player_id,
                                amount: stake.amount,
                                rank: 1,
                                percentage: 100 / wagering_state.player_stakes.len() as u8,
                            })
                            .collect())
                    }
                }
                WageringType::SplitPot {
                    winner_percentage,
                    runner_up_percentage,
                } => {
                    // This would need game-specific logic to determine runner-up
                    // For now, fall back to winner-takes-all
                    self.calculate_payouts(game_state)
                } // WageringType::Tournament { payouts } => {
                  //     // This would need game-specific logic to determine rankings
                  //     // For now, fall back to winner-takes-all
                  //     self.calculate_payouts(game_state)
                  // }
                  // WageringType::Custom { logic: _ } => {
                  //     Err("Custom payout logic must be implemented by the game".to_string())
                  // }
            }
        } else {
            Ok(vec![]) // No wagering
        }
    }
    //validate stake(bet)
    fn validate_stake(
        &self,
        game_state: &GameState,
        player_id: PlayerId,
        amount: u64,
    ) -> Result<(), String> {
        if let Some(wagering_state) = &game_state.wagering_state {
            // Check if player is in the game
            if !game_state.players.contains(&player_id) {
                return Err("Player not in game".to_string());
            }

            // Check if player already committed
            if wagering_state.player_stakes.contains_key(&player_id) {
                return Err("Player already committed stake".to_string());
            }

            // Check stake amount
            if amount < wagering_state.config.min_stake {
                return Err(format!(
                    "Stake too low. Minimum: {}",
                    wagering_state.config.min_stake
                ));
            }

            if let Some(max_stake) = wagering_state.config.max_stake {
                if amount > max_stake {
                    return Err(format!("Stake too high. Maximum: {}", max_stake));
                }
            }

            // Check equal stakes requirement
            if wagering_state.config.equal_stakes && !wagering_state.player_stakes.is_empty() {
                let first_stake = wagering_state.player_stakes.values().next().unwrap().amount;
                if amount != first_stake {
                    return Err(format!(
                        "All players must stake the same amount. Expected: {}",
                        first_stake
                    ));
                }
            }
            Ok(())
        } else {
            Err("Game does not support wagering".to_string())
        }
    }

    //serialization, deserialization and formatting
}

pub struct GameEngineRegistry {
    engines: HashMap<GameTypeId, Box<dyn GameEngine>>,
}

impl GameEngineRegistry {
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    pub fn register_engine(&mut self, engine: Box<dyn GameEngine>) {
        let game_type_id = engine.game_type_id();
        self.engines.insert(game_type_id, engine);
    }

    pub fn get_engine(&self, game_type_id: &GameTypeId) -> Option<&dyn GameEngine> {
        self.engines.get(game_type_id).map(|e| e.as_ref())
    }

    //List all game types
}

pub struct GameStateManager {
    registry: GameEngineRegistry,
    active_games: HashMap<GameInstanceId, GameState>,
}

impl GameStateManager {
    pub fn new(registry: GameEngineRegistry) -> Self {
        Self {
            registry,
            active_games: HashMap::new(),
        }
    }

    //Create a new game instance
    pub fn create_game(
        &mut self,
        config: &GameConfig,
        players: &[PlayerId],
    ) -> Result<GameInstanceId, String> {
        let engine = self
            .registry
            .get_engine(&config.game_type_id)
            .ok_or_else(|| format!("Game type '{}' not found", config.game_type_id))?;

        let game_state = engine.create_game(config, players)?;
        let game_instance_id = game_state.game_instance_id;

        self.active_games.insert(game_instance_id, game_state);
        Ok(game_instance_id)
    }

    /// Process a game move
    pub fn process_move(&mut self, game_move: &GameMove) -> Result<GameActionResult, String> {
        let game_state = self
            .active_games
            .get(&game_move.game_instance_id)
            .ok_or_else(|| "Game not found".to_string())?;

        let engine = self
            .registry
            .get_engine(&game_state.game_type_id)
            .ok_or_else(|| "Game engine not found".to_string())?;

        // Validate the move
        let validation_result = engine.validate_move(game_state, game_move);
        if let GameActionResult::Failure(_) = validation_result {
            return Ok(validation_result);
        }

        // Apply the move
        let mut new_state = engine.apply_move(game_state, game_move)?;

        if let Some(winner) = engine.check_game_end(&game_state) {
            new_state.is_finished = true;
            new_state.winner = Some(winner);

            // Calculate payouts if wagering is enabled
            if let Some(wagering_state) = &new_state.wagering_state {
                if wagering_state.stakes_committed {
                    let payouts = engine.calculate_payouts(&new_state)?;
                    new_state.wagering_state.as_mut().unwrap().payouts = Some(payouts);
                }
            }
        }
        // Update the game state
        self.active_games
            .insert(game_move.game_instance_id, new_state);

        Ok(validation_result)
    }

    pub fn commit_stake(
        &mut self,
        game_instance_id: &GameInstanceId,
        player_id: PlayerId,
        amount: u64,
    ) -> Result<(), String> {
        let game_state = self
            .active_games
            .get_mut(game_instance_id)
            .ok_or_else(|| "Game not found".to_string())?;

        let engine = self
            .registry
            .get_engine(&game_state.game_type_id)
            .ok_or_else(|| "Game engine not found".to_string())?;

        //validate the stake
        engine.validate_stake(game_state, player_id, amount)?;

        // Add the stake
        if let Some(wagering_state) = &mut game_state.wagering_state {
            let player_stake = PlayerStake {
                player_id,
                amount,
                token_mint: None, // Default to SOL
                committed: true,
                committed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            wagering_state.player_stakes.insert(player_id, player_stake);
            wagering_state.total_pot += amount;

            // Check if all players have committed
            if wagering_state.player_stakes.len() == game_state.players.len() {
                wagering_state.stakes_committed = true;
            }
        } else {
            return Err("Game does not support wagering".to_string());
        }

        Ok(())
    }

    // current game state
    /// Get the current state of a game
    pub fn get_game_state(&self, game_instance_id: &GameInstanceId) -> Option<&GameState> {
        self.active_games.get(game_instance_id)
    }

    /// Get the current state of a game (mutable)
    pub fn get_game_state_mut(
        &mut self,
        game_instance_id: &GameInstanceId,
    ) -> Option<&mut GameState> {
        self.active_games.get_mut(game_instance_id)
    }

    //get active games
    //remove finished game
    //get registry
    /// Get registry for external access
    pub fn get_registry(&self) -> &GameEngineRegistry {
        &self.registry
    }
}
