use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChestPiece {
    Pawn,
    Scout,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Slot {
    pub piece: ChestPiece,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub slots: [[Slot; 8]; 8],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub id: Uuid,
    pub player1_remaining_moves: u64,
    pub player1_move_increment_countdown: u64,
    pub player2_remaining_moves: u64,
    pub player2_move_increment_countdown: u64,
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

impl Default for Slot {
    fn default() -> Self {
        Slot {
            piece: ChestPiece::default(),
        }
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.piece)
    }
}

impl Default for Board {
    fn default() -> Self {
        Board {
            slots: [[Slot::default(); 8]; 8],
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.slots {
            for slot in row {
                write!(f, "{} ", slot)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Game {
    pub fn new() -> Self {
        Game {
            id: Uuid::new_v4(),
            ..Default::default()
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
