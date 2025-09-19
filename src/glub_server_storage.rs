use crate::glub_server::*;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct GameStorage {
    games: HashMap<Uuid, GameState>,
    queue: Vec<QueuedPlayer>,
}

#[derive(Debug)]
pub struct QueuedPlayer {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub struct GameState {
    pub game: Game,
    pub board: ExtendedBoard,
    pub player1: PlayerInfo,
    pub player2: PlayerInfo,
    pub created_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
    pub color: PlayerColor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerColor {
    White,
    Black,
}

#[derive(Serialize)]
pub struct FoggedBoard {
    pub slots: [[Option<VisibleSlot>; 8]; 8],
    pub your_color: PlayerColor,
}

#[derive(Serialize, Clone, Debug)]
pub struct VisibleSlot {
    pub piece: ChestPiece,
    pub color: PlayerColor,
}

impl GameStorage {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
            queue: Vec::new(),
        }
    }

    pub fn join_queue(&mut self, player_name: String) -> Result<crate::JoinQueueResponse, String> {
        let player_id = Uuid::new_v4();

        // Check if there's already a player waiting
        if let Some(waiting_player) = self.queue.pop() {
            // Create a new game with both players
            let game_id = self.create_game(
                waiting_player,
                QueuedPlayer {
                    id: player_id,
                    name: player_name,
                },
            )?;

            Ok(crate::JoinQueueResponse {
                player_id,
                game_id: Some(game_id),
                message: "Game started!".to_string(),
            })
        } else {
            // Add to queue
            self.queue.push(QueuedPlayer {
                id: player_id,
                name: player_name,
            });

            Ok(crate::JoinQueueResponse {
                player_id,
                game_id: None,
                message: "Added to queue, waiting for opponent...".to_string(),
            })
        }
    }

    fn create_game(
        &mut self,
        player1: QueuedPlayer,
        player2: QueuedPlayer,
    ) -> Result<Uuid, String> {
        let game_id = Uuid::new_v4();
        let mut board = ExtendedBoard::new();
        board.setup_initial_position();

        let game_state = GameState {
            game: Game {
                id: game_id,
                player1_remaining_moves: 1,          // Start with 1 move
                player1_move_increment_countdown: 3, // 3 seconds until next move point
                player2_remaining_moves: 1,
                player2_move_increment_countdown: 3,
            },
            board,
            player1: PlayerInfo {
                id: player1.id,
                name: player1.name,
                color: PlayerColor::White,
            },
            player2: PlayerInfo {
                id: player2.id,
                name: player2.name,
                color: PlayerColor::Black,
            },
            created_at: std::time::Instant::now(),
        };

        self.games.insert(game_id, game_state);
        Ok(game_id)
    }

    pub fn get_fogged_board(&self, game_id: Uuid, player_id: Uuid) -> Result<FoggedBoard, String> {
        let game_state = self.games.get(&game_id).ok_or("Game not found")?;

        let player_color = if game_state.player1.id == player_id {
            game_state.player1.color.clone()
        } else if game_state.player2.id == player_id {
            game_state.player2.color.clone()
        } else {
            return Err("Player not in this game".to_string());
        };

        let visible_positions = game_state.board.get_visible_positions(&player_color);
        let mut fogged_slots: [[Option<VisibleSlot>; 8]; 8] = Default::default();

        for row in 0..8 {
            for col in 0..8 {
                if visible_positions.contains(&(row, col)) {
                    if let Some(piece_info) = &game_state.board.slots[row][col] {
                        fogged_slots[row][col] = Some(VisibleSlot {
                            piece: piece_info.piece,
                            color: piece_info.color.clone(),
                        });
                    }
                }
            }
        }

        Ok(FoggedBoard {
            slots: fogged_slots,
            your_color: player_color,
        })
    }

    pub fn make_move(
        &mut self,
        game_id: Uuid,
        move_req: crate::MoveRequest,
    ) -> Result<crate::MoveResponse, String> {
        let game_state = self.games.get_mut(&game_id).ok_or("Game not found")?;

        let (is_player1, remaining_moves) = if game_state.player1.id == move_req.player_id {
            (true, game_state.game.player1_remaining_moves)
        } else if game_state.player2.id == move_req.player_id {
            (false, game_state.game.player2_remaining_moves)
        } else {
            return Err("Player not in this game".to_string());
        };

        if remaining_moves == 0 {
            return Ok(crate::MoveResponse {
                success: false,
                message: "No moves remaining".to_string(),
                remaining_moves: 0,
            });
        }

        let player_color = if is_player1 {
            &game_state.player1.color
        } else {
            &game_state.player2.color
        };

        // Validate and execute the move
        match game_state
            .board
            .make_move(move_req.from, move_req.to, player_color)
        {
            Ok(_) => {
                // Consume a move point
                if is_player1 {
                    game_state.game.player1_remaining_moves -= 1;
                } else {
                    game_state.game.player2_remaining_moves -= 1;
                }

                let remaining = if is_player1 {
                    game_state.game.player1_remaining_moves
                } else {
                    game_state.game.player2_remaining_moves
                };

                Ok(crate::MoveResponse {
                    success: true,
                    message: "Move successful".to_string(),
                    remaining_moves: remaining,
                })
            }
            Err(e) => Ok(crate::MoveResponse {
                success: false,
                message: e,
                remaining_moves: remaining_moves,
            }),
        }
    }

    pub fn get_game_status(&self, game_id: Uuid) -> Result<crate::GameStatus, String> {
        let game_state = self.games.get(&game_id).ok_or("Game not found")?;

        Ok(crate::GameStatus {
            game_id,
            player1_moves: game_state.game.player1_remaining_moves,
            player2_moves: game_state.game.player2_remaining_moves,
            current_turn: None, // In this system, both players can move simultaneously
        })
    }

    pub fn increment_moves(&mut self) {
        for game_state in self.games.values_mut() {
            // Player 1 move increment
            if game_state.game.player1_move_increment_countdown > 0 {
                game_state.game.player1_move_increment_countdown -= 1;
            } else {
                game_state.game.player1_remaining_moves = std::cmp::min(
                    game_state.game.player1_remaining_moves + 1,
                    5, // Max 5 moves stored
                );
                game_state.game.player1_move_increment_countdown = 3; // Reset to 3 seconds
            }

            // Player 2 move increment
            if game_state.game.player2_move_increment_countdown > 0 {
                game_state.game.player2_move_increment_countdown -= 1;
            } else {
                game_state.game.player2_remaining_moves = std::cmp::min(
                    game_state.game.player2_remaining_moves + 1,
                    5, // Max 5 moves stored
                );
                game_state.game.player2_move_increment_countdown = 3; // Reset to 3 seconds
            }
        }
    }
}

// Implement Serialize for PlayerColor
impl Serialize for PlayerColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PlayerColor::White => serializer.serialize_str("white"),
            PlayerColor::Black => serializer.serialize_str("black"),
        }
    }
}
