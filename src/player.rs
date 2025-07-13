use crate::game::GameState;
use crate::interpreter::Interpreter;
use crate::strategy::Strategy;
use anyhow::Result;
use tokio::time::{sleep, Duration};

/// Player orchestrates the game by connecting interpreter, state, and strategy
pub struct Player<I: Interpreter, S: Strategy> {
    interpreter: I,
    strategy: S,
    game_state: GameState,
    display_output: bool,
    max_turns: usize,
    turn_count: usize,
}

impl<I: Interpreter, S: Strategy> Player<I, S> {
    pub fn new(interpreter: I, strategy: S, display_output: bool) -> Self {
        Self {
            interpreter,
            strategy,
            game_state: GameState::new(),
            display_output,
            max_turns: 1000, // Prevent infinite loops
            turn_count: 0,
        }
    }
    
    /// Set the maximum number of turns to prevent infinite loops
    pub fn set_max_turns(&mut self, max_turns: usize) {
        self.max_turns = max_turns;
    }
    
    /// Play one complete game
    pub async fn play_game(&mut self, program_path: &str) -> Result<GameResult> {
        log::info!("Starting game with strategy: {}", self.strategy.name());
        
        // Launch the interpreter
        self.interpreter.launch(program_path).await?;
        
        // Reset strategy and game state
        self.strategy.reset();
        self.game_state = GameState::new();
        self.turn_count = 0;
        
        // Main game loop
        while self.interpreter.is_running() && self.turn_count < self.max_turns {
            // Read output from interpreter
            let output = self.interpreter.read_until_prompt().await?;
            
            if output.is_empty() {
                log::warn!("No output received from interpreter");
                sleep(Duration::from_millis(100)).await;
                continue;
            }
            
            // Display output if requested
            if self.display_output {
                for line in &output {
                    println!("{}", line);
                }
            }
            
            // Update game state
            self.game_state.update(&output)?;
            
            // Display current game status (unless it's the first turn without state)
            if self.turn_count > 0 || self.game_state.stardate.is_some() {
                self.game_state.display_status();
            }
            
            // Check for game end conditions
            if self.is_game_over(&output) {
                let result = self.determine_game_result(&output);
                log::info!("Game ended: {:?}", result);
                // Try to terminate interpreter gracefully to allow coverage data saving
                if let Err(e) = self.interpreter.terminate().await {
                    log::warn!("Failed to terminate interpreter gracefully: {}", e);
                }
                return Ok(result);
            }
            
            // Get next command from strategy
            let command = self.strategy.get_command(&self.game_state)?;
            log::debug!("Sending command: {}", command);
            
            // DEBUG: Check for blank commands and provide detailed info
            if command.trim().is_empty() {
                // Check if this is an expected blank command response
                let current_prompt = self.game_state.get_current_prompt().unwrap_or("").trim();
                let is_expected_blank = match current_prompt {
                    "PLEASE ENTER" => true,
                    "ENTER ONE OF THE FOLLOWING:" => true,
                    p if p.contains("SHIELDS NOW AT") && p.contains("UNITS PER YOUR COMMAND") => true,
                    p if p.contains("DEFLECTOR CONTROL ROOM REPORT") => true,
                    p if p.contains("NOW ENTERING") && p.contains("QUADRANT") => true,
                    _ => false,
                };
                
                if !is_expected_blank {
                    eprintln!("🚨 DEBUG: About to send blank command!");
                    eprintln!("  Current prompt: {:?}", self.game_state.get_current_prompt());
                    eprintln!("  Last 5 output lines:");
                    for (i, line) in self.game_state.last_output.iter().rev().take(5).enumerate() {
                        eprintln!("    -{}: {}", i+1, line);
                    }
                    eprintln!("  Game state: stardate={:?}, condition={:?}", 
                              self.game_state.stardate, self.game_state.condition);
                    eprintln!("  Strategy: {}", self.strategy.name());
                    std::process::exit(1);
                }
            }
            
            // Display command if output is enabled
            if self.display_output {
            //     if command.trim().is_empty() {
            //         println!("🤖 TrekBot sends: [ENTER]");
            //     } else {
                    println!("🤖 TrekBot sends: {}", command);
                // }
            }
            
            // Send command to interpreter
            self.interpreter.send_command(&command).await?;
            
            self.turn_count += 1;
            
            // Small delay to prevent overwhelming the interpreter
            sleep(Duration::from_millis(10)).await;
        }
        
        if self.turn_count >= self.max_turns {
            log::warn!("Game ended due to max turns limit");
            // Try to terminate interpreter gracefully to allow coverage data saving
            if let Err(e) = self.interpreter.terminate().await {
                log::warn!("Failed to terminate interpreter gracefully: {}", e);
            }
            Ok(GameResult::MaxTurnsReached)
        } else {
            log::info!("Game ended - interpreter stopped");
            Ok(GameResult::InterpreterStopped)
        }
    }
    
    /// Check if the game has ended based on output
    fn is_game_over(&self, output: &[String]) -> bool {
        for line in output {
            let line = line.to_uppercase();
            if line.contains("MISSION ACCOMPLISHED") 
                || line.contains("YOU HAVE BEEN KILLED") 
                || line.contains("GAME OVER") 
                || line.contains("FEDERATION DESTROYED")
                || line.contains("TIME HAS RUN OUT") {
                return true;
            }
        }
        false
    }
    
    /// Determine the game result based on output
    fn determine_game_result(&self, output: &[String]) -> GameResult {
        for line in output {
            let line = line.to_uppercase();
            if line.contains("MISSION ACCOMPLISHED") {
                return GameResult::Victory;
            } else if line.contains("YOU HAVE BEEN KILLED") {
                return GameResult::Destroyed;
            } else if line.contains("TIME HAS RUN OUT") {
                return GameResult::TimeUp;
            } else if line.contains("FEDERATION DESTROYED") {
                return GameResult::FederationDestroyed;
            }
        }
        GameResult::Unknown
    }
    
    /// Get the current game state
    pub fn get_game_state(&self) -> &GameState {
        &self.game_state
    }
    
    /// Get the current turn count
    pub fn get_turn_count(&self) -> usize {
        self.turn_count
    }
}

impl<I: Interpreter, S: Strategy> Drop for Player<I, S> {
    fn drop(&mut self) {
        // Attempt to terminate interpreter on drop
        // We can't use async here, so we'll spawn a task
        tokio::spawn(async {
            // This is a best effort cleanup
        });
    }
}

/// Result of a game session
#[derive(Debug, Clone, PartialEq)]
pub enum GameResult {
    Victory,
    Destroyed,
    TimeUp,
    FederationDestroyed,
    MaxTurnsReached,
    InterpreterStopped,
    Unknown,
}

impl GameResult {
    pub fn is_success(&self) -> bool {
        matches!(self, GameResult::Victory)
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            GameResult::Victory => "Mission accomplished! All Klingons destroyed.",
            GameResult::Destroyed => "Enterprise destroyed in battle.",
            GameResult::TimeUp => "Time ran out before mission completion.",
            GameResult::FederationDestroyed => "Federation headquarters destroyed.",
            GameResult::MaxTurnsReached => "Game ended due to turn limit.",
            GameResult::InterpreterStopped => "Interpreter process stopped.",
            GameResult::Unknown => "Game ended for unknown reasons.",
        }
    }
}

/// Statistics for multiple games
#[derive(Debug, Clone)]
pub struct GameStats {
    pub total_games: usize,
    pub victories: usize,
    pub destroyed: usize,
    pub time_up: usize,
    pub other: usize,
    pub avg_turns: f64,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            total_games: 0,
            victories: 0,
            destroyed: 0,
            time_up: 0,
            other: 0,
            avg_turns: 0.0,
        }
    }
    
    pub fn add_game(&mut self, result: GameResult, turns: usize) {
        self.total_games += 1;
        
        match result {
            GameResult::Victory => self.victories += 1,
            GameResult::Destroyed => self.destroyed += 1,
            GameResult::TimeUp => self.time_up += 1,
            _ => self.other += 1,
        }
        
        // Update average turns
        self.avg_turns = ((self.avg_turns * (self.total_games - 1) as f64) + turns as f64) / self.total_games as f64;
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_games == 0 {
            0.0
        } else {
            self.victories as f64 / self.total_games as f64
        }
    }
    
    pub fn print_summary(&self) {
        println!("=== Game Statistics ===");
        println!("Total games: {}", self.total_games);
        println!("Victories: {} ({:.1}%)", self.victories, self.success_rate() * 100.0);
        println!("Destroyed: {} ({:.1}%)", self.destroyed, self.destroyed as f64 / self.total_games as f64 * 100.0);
        println!("Time up: {} ({:.1}%)", self.time_up, self.time_up as f64 / self.total_games as f64 * 100.0);
        println!("Other: {} ({:.1}%)", self.other, self.other as f64 / self.total_games as f64 * 100.0);
        println!("Average turns: {:.1}", self.avg_turns);
    }
}

impl Default for GameStats {
    fn default() -> Self {
        Self::new()
    }
} 