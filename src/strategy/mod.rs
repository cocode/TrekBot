use crate::game::GameState;
use anyhow::Result;

pub mod random;
pub mod cheat;

pub use random::*;
pub use cheat::*;

/// Trait for different game playing strategies
pub trait Strategy {
    /// Get the next command to send to the game based on the current state
    fn get_command(&mut self, game_state: &GameState) -> Result<String>;
    
    /// Reset the strategy state (e.g., between games)
    fn reset(&mut self);
    
    /// Get the name of this strategy
    fn name(&self) -> &'static str;
}

/// Command types that can be sent to the game
#[derive(Debug, Clone)]
pub enum Command {
    Navigation,
    ShortRangeScan,
    LongRangeScan,
    Phasers,
    Torpedoes,
    Shields,
    DamageControl,
    Computer,
    Quit,
}

impl Command {
    pub fn to_string(&self) -> String {
        match self {
            Command::Navigation => "NAV".to_string(),
            Command::ShortRangeScan => "SRS".to_string(),
            Command::LongRangeScan => "LRS".to_string(),
            Command::Phasers => "PHA".to_string(),
            Command::Torpedoes => "TOR".to_string(),
            Command::Shields => "SHE".to_string(),
            Command::DamageControl => "DAM".to_string(),
            Command::Computer => "COM".to_string(),
            Command::Quit => "XXX".to_string(),
        }
    }
}

/// Helper function to generate random commands
pub fn random_command() -> Command {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let commands = vec![
        Command::Navigation,
        Command::ShortRangeScan,
        Command::LongRangeScan,
        Command::Phasers,
        Command::Torpedoes,
        Command::Shields,
        Command::DamageControl,
        Command::Computer,
        // Command::Quit, // Don't include quit in random selection
    ];
    
    let index = rng.gen_range(0..commands.len());
    commands[index].clone()
} 