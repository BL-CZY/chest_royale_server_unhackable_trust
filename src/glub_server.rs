use crate::glub_server_storage::PlayerColor;
use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ChestPiece {
    Pawn,
    Scout,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedSlot {
    pub piece: ChestPiece,
    pub color: PlayerColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedBoard {
    pub slots: [[Option<ExtendedSlot>; 8]; 8],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub id: Uuid,
    pub player1_remaining_moves: u64,
    pub player1_move_increment_countdown: u64,
    pub player2_remaining_moves: u64,
    pub player2_move_increment_countdown: u64,
}

impl ExtendedBoard {
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
        }
    }

    pub fn setup_initial_position(&mut self) {
        // Clear the board first
        self.slots = Default::default();

        // Setup white pieces (bottom rows)
        // Pawns on row 1
        for col in 0..8 {
            self.slots[1][col] = Some(ExtendedSlot {
                piece: ChestPiece::Pawn,
                color: PlayerColor::White,
            });
        }

        // Major pieces on row 0
        let white_pieces = [
            ChestPiece::Rook,
            ChestPiece::Knight,
            ChestPiece::Bishop,
            ChestPiece::Queen,
            ChestPiece::King,
            ChestPiece::Bishop,
            ChestPiece::Scout,
            ChestPiece::Rook,
        ];

        for (col, &piece) in white_pieces.iter().enumerate() {
            self.slots[0][col] = Some(ExtendedSlot {
                piece,
                color: PlayerColor::White,
            });
        }

        // Setup black pieces (top rows)
        // Pawns on row 6
        for col in 0..8 {
            self.slots[6][col] = Some(ExtendedSlot {
                piece: ChestPiece::Pawn,
                color: PlayerColor::Black,
            });
        }

        // Major pieces on row 7
        let black_pieces = [
            ChestPiece::Rook,
            ChestPiece::Scout,
            ChestPiece::Bishop,
            ChestPiece::Queen,
            ChestPiece::King,
            ChestPiece::Bishop,
            ChestPiece::Knight,
            ChestPiece::Rook,
        ];

        for (col, &piece) in black_pieces.iter().enumerate() {
            self.slots[7][col] = Some(ExtendedSlot {
                piece,
                color: PlayerColor::Black,
            });
        }
    }

    pub fn get_visible_positions(&self, player_color: &PlayerColor) -> HashSet<(usize, usize)> {
        let mut visible = HashSet::new();

        // Find all pieces belonging to the player
        for row in 0..8 {
            for col in 0..8 {
                if let Some(slot) = &self.slots[row][col] {
                    if slot.color == *player_color {
                        // Add the piece's own position
                        visible.insert((row, col));

                        // Add positions this piece can see
                        let sight_range = match slot.piece {
                            ChestPiece::Scout => 3,
                            _ => 1,
                        };

                        self.add_visible_positions(&mut visible, row, col, sight_range);
                    }
                }
            }
        }

        visible
    }

    fn add_visible_positions(
        &self,
        visible: &mut HashSet<(usize, usize)>,
        center_row: usize,
        center_col: usize,
        range: usize,
    ) {
        let center_row = center_row as i32;
        let center_col = center_col as i32;

        for dr in -(range as i32)..=(range as i32) {
            for dc in -(range as i32)..=(range as i32) {
                let new_row = center_row + dr;
                let new_col = center_col + dc;

                if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                    let distance = ((dr.abs() as f64).powi(2) + (dc.abs() as f64).powi(2)).sqrt();
                    if distance <= range as f64 {
                        visible.insert((new_row as usize, new_col as usize));
                    }
                }
            }
        }
    }

    pub fn make_move(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
        player_color: &PlayerColor,
    ) -> Result<(), String> {
        let (from_row, from_col) = from;
        let (to_row, to_col) = to;

        // Validate coordinates
        if from_row >= 8 || from_col >= 8 || to_row >= 8 || to_col >= 8 {
            return Err("Invalid coordinates".to_string());
        }

        // Check if there's a piece at the from position
        let piece_info = match &self.slots[from_row][from_col] {
            Some(slot) => slot.clone(),
            None => return Err("No piece at source position".to_string()),
        };

        // Check if the piece belongs to the player
        if piece_info.color != *player_color {
            return Err("Not your piece".to_string());
        }

        // Check if the move is valid for this piece type
        if !self.is_valid_move(&piece_info, from, to) {
            return Err("Invalid move for this piece".to_string());
        }

        // Special rule: Scouts cannot capture
        if piece_info.piece == ChestPiece::Scout {
            if self.slots[to_row][to_col].is_some() {
                return Err("Scouts cannot capture pieces".to_string());
            }
        }

        // Check if destination has own piece
        if let Some(dest_piece) = &self.slots[to_row][to_col] {
            if dest_piece.color == *player_color {
                return Err("Cannot capture your own piece".to_string());
            }
        }

        // Execute the move
        self.slots[from_row][from_col] = None;
        self.slots[to_row][to_col] = Some(piece_info);

        Ok(())
    }

    fn is_valid_move(
        &self,
        piece_info: &ExtendedSlot,
        from: (usize, usize),
        to: (usize, usize),
    ) -> bool {
        let (from_row, from_col) = (from.0 as i32, from.1 as i32);
        let (to_row, to_col) = (to.0 as i32, to.1 as i32);
        let dr = to_row - from_row;
        let dc = to_col - from_col;

        match piece_info.piece {
            ChestPiece::Pawn => {
                let forward = if piece_info.color == PlayerColor::White {
                    1
                } else {
                    -1
                };

                // Forward move
                if dc == 0 && dr == forward {
                    return self.slots[to.0][to.1].is_none();
                }

                // Diagonal capture
                if dc.abs() == 1 && dr == forward {
                    return self.slots[to.0][to.1].is_some();
                }

                false
            }

            ChestPiece::Scout => {
                // Scouts can move 1 or 2 tiles in any direction
                let distance = ((dr.abs() as f64).powi(2) + (dc.abs() as f64).powi(2)).sqrt();
                distance <= 2.0 && distance >= 1.0
            }

            ChestPiece::Rook => (dr == 0 || dc == 0) && self.is_path_clear(from, to),

            ChestPiece::Knight => {
                (dr.abs() == 2 && dc.abs() == 1) || (dr.abs() == 1 && dc.abs() == 2)
            }

            ChestPiece::Bishop => dr.abs() == dc.abs() && self.is_path_clear(from, to),

            ChestPiece::Queen => {
                (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && self.is_path_clear(from, to)
            }

            ChestPiece::King => dr.abs() <= 1 && dc.abs() <= 1 && (dr != 0 || dc != 0),
        }
    }

    fn is_path_clear(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (from_row, from_col) = (from.0 as i32, from.1 as i32);
        let (to_row, to_col) = (to.0 as i32, to.1 as i32);

        let dr = (to_row - from_row).signum();
        let dc = (to_col - from_col).signum();

        let mut current_row = from_row + dr;
        let mut current_col = from_col + dc;

        while current_row != to_row || current_col != to_col {
            if current_row < 0 || current_row >= 8 || current_col < 0 || current_col >= 8 {
                return false;
            }

            if self.slots[current_row as usize][current_col as usize].is_some() {
                return false;
            }

            current_row += dr;
            current_col += dc;
        }

        true
    }
}

// Additional trait implementations
impl Default for ChestPiece {
    fn default() -> Self {
        ChestPiece::Pawn
    }
}

impl std::fmt::Display for ChestPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            ChestPiece::Pawn => "♟",
            ChestPiece::Scout => "◊", // Custom symbol for Scout
            ChestPiece::Rook => "♜",
            ChestPiece::Knight => "♞",
            ChestPiece::Bishop => "♝",
            ChestPiece::Queen => "♛",
            ChestPiece::King => "♚",
        };
        write!(f, "{}", symbol)
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            id: Uuid::new_v4(),
            player1_remaining_moves: 1,
            player1_move_increment_countdown: 3,
            player2_remaining_moves: 1,
            player2_move_increment_countdown: 3,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game({})", self.id)
    }
}
