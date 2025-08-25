// Possible outcomes for Rock, Paper, Scissors game

// Draw
// Both Rock, Paper or Scissors

// Win
// Rock beats Scissors
// Paper beats Rock
// Scissors beats Paper

// Lose
// Rock loses to Paper
// Paper loses to Scissors
// Scissors loses to Rock

use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
// use solana_sdk::borsh

use crate::game_traits::*;

/// RPS choice enumeration(should serve as move )
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum RPSChoice {
    Rock,
    Paper,
    Scissors,
}

/// RPS game state
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RPSGameState {
    pub players: [PlayerId; 2],
    // pub commitments: HashMap<PlayerId, Vec<u8>>, // Hash of choice + nonce
    pub choices: HashMap<PlayerId, RPSChoice>,
    pub nonces: HashMap<PlayerId, Vec<u8>>,
    // pub phase: RPSPhase,
    pub winner: Option<PlayerId>,
}

/// Game phases for RPS
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum RPSPhase {
    Commitment, // Players commit their choices
    Reveal,     // Players reveal their choices
    Finished,   // Game is complete
}

pub struct RPSEngine;

impl RPSEngine {
    pub fn new() -> Self {
        Self
    }

    fn determine_winner(&self, rps_state: &RPSGameState) -> Option<PlayerId> {
        let [player1, player2] = &rps_state.players;
        let choice1 = rps_state.choices.get(player1).unwrap();
        let choice2 = rps_state.choices.get(player2).unwrap();

        match (choice1, choice2) {
            (RPSChoice::Rock, RPSChoice::Scissors) => Some(*player1),
            (RPSChoice::Paper, RPSChoice::Rock) => Some(*player1),
            (RPSChoice::Scissors, RPSChoice::Paper) => Some(*player1),
            (RPSChoice::Scissors, RPSChoice::Rock) => Some(*player2),
            (RPSChoice::Rock, RPSChoice::Paper) => Some(*player2),
            (RPSChoice::Paper, RPSChoice::Scissors) => Some(*player2),
            _ => None, // Tie
        }
    }
}

impl GameEngine for RPSEngine {
    fn game_type_id(&self) -> GameTypeId {
        "rock_paper_scissors".to_string()
    }

    fn display_name(&self) -> String {
        "Rock-Paper-Scissors".to_string()
    }

    fn description(&self) -> String {
        "Classic Rock-Paper-Scissors with commitment scheme for fair play".to_string()
    }

    fn max_players(&self) -> u8 {
        2
    }

    fn min_players(&self) -> u8 {
        2
    }

    fn create_game(&self, config: &GameConfig, players: &[PlayerId]) -> Result<GameState, String> {
        if players.len() != 2 {
            return Err("Rock-Paper-Scissors requires exactly 2 players".to_string());
        }

        let game_state = RPSGameState {
            players: [players[0], players[1]],
            // commitments: HashMap::new(),
            choices: HashMap::new(),
            nonces: HashMap::new(),
            // phase: RPSPhase::Commitment,
            winner: None,
        };

        // let state = game_state.down
        let state_data = game_state.try_to_vec().unwrap();

        Ok(GameState {
            game_instance_id: Pubkey::new_unique(),
            game_type_id: self.game_type_id(),
            players: players.to_vec(),
            current_player: None,
            state_data,
            move_history: Vec::new(),
            is_finished: false,
            winner: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            wagering_state: None,
            last_activity: 0,
        })
    }

    fn validate_move(&self, game_state: &GameState, game_move: &GameMove) -> GameActionResult {
        if game_state.is_finished {
            return GameActionResult::Failure("Game is already finished".to_string());
        }

        let rps_state = RPSGameState::try_from_slice(&game_state.state_data).unwrap(); // add error handling
        let move_data = RPSChoice::try_from_slice(&game_move.move_data).unwrap();

        // Check if player is in the game
        if !rps_state.players.contains(&game_move.player_id) {
            return GameActionResult::Failure("Player not in game".to_string());
        }

        if rps_state.choices.contains_key(&game_move.player_id) {
            return GameActionResult::Failure("Player already played".to_string());
        }
        //add more checks for validation
        GameActionResult::Success
    }

    fn apply_move(
        &self,
        game_state: &GameState,
        game_move: &GameMove,
    ) -> Result<GameState, String> {
        let mut rps_state = RPSGameState::try_from_slice(&game_state.state_data).unwrap(); // add error handling
        let move_data = RPSChoice::try_from_slice(&game_move.move_data).unwrap();

        rps_state.choices.insert(game_move.player_id, move_data);

        if rps_state.choices.len() == 2 {
            //update winner and finished state in state

            if let Some(winner) = self.determine_winner(&rps_state) {
                rps_state.winner = Some(winner);
            };
        }

        let mut new_state = game_state.clone();
        new_state.state_data = rps_state.try_to_vec().unwrap();
        new_state.move_history.push(game_move.clone());
        new_state.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        new_state.is_finished = true;
        new_state.winner = rps_state.winner;

        Ok(new_state)
    }

    fn check_game_end(&self, game_state: &GameState) -> Option<PlayerId> {
        if game_state.is_finished {
            game_state.winner
        } else {
            None
        }
    }
}
