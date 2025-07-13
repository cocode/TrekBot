use crate::game::{GameState, parse_energy_available, parse_warp_factor_range};
use crate::strategy::{Strategy, random_command};
use anyhow::Result;
use rand::Rng;

/// Random strategy implementation that plays the game randomly
/// This is similar to the original Python RandomStrategy but designed to be legal ~90% of the time
pub struct RandomStrategy {
    rng: rand::rngs::ThreadRng,
    first_turn: bool,
}

impl RandomStrategy {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            first_turn: true,
        }
    }
    
    /// Handle the main command prompt
    fn handle_command_prompt(&mut self, game_state: &GameState) -> Result<String> {
        // // If this is the first turn, set shields to a random value between 0-1000
        // if self.first_turn {
        //     self.first_turn = false;
        //     return Ok("SHE".to_string()); // Shield command
        // }
        //
        // Check if we're in a dangerous situation and need shields
        let is_dangerous = game_state.last_output.iter().any(|output| {
            output.contains("CONDITION RED") || 
            output.contains("COMBAT AREA") ||
            output.contains("SHIELDS DANGEROUSLY LOW") ||
            output.contains("UNIT HIT ON ENTERPRISE")
        });
        
        // If we're in danger and shields are low, prioritize shield commands
        if is_dangerous {
            // 50% chance to use shields when in danger
            if self.rng.gen_bool(0.5) {
                return Ok("SHE".to_string()); // Shield command
            }
            // 30% chance to use phasers when in danger
            if self.rng.gen_bool(0.3) {
                return Ok("PHA".to_string()); // Phaser command
            }
        }
        
        // Otherwise use random command
        Ok(random_command().to_string())
    }
    
    /// Handle torpedo course prompt
    fn handle_torpedo_course(&mut self, _game_state: &GameState) -> Result<String> {
        let course = self.rng.gen_range(1..10);
        Ok(course.to_string())
    }
    
    /// Handle computer command prompt
    fn handle_computer_command(&mut self, _game_state: &GameState) -> Result<String> {
        // Super star trek has a bug - anything larger than 5 crashes
        let command = self.rng.gen_range(0..6);
        Ok(command.to_string())
    }
    
    /// Handle course selection prompt
    fn handle_course_prompt(&mut self, _game_state: &GameState) -> Result<String> {
        let course = self.rng.gen_range(1..10);
        Ok(course.to_string())
    }
    
    /// Handle shield units prompt
    fn handle_shield_units(&mut self, game_state: &GameState) -> Result<String> {
        // Try to extract energy from the last output
        let energy = if let Some(last_output) = game_state.last_output.last() {
            parse_energy_available(last_output).unwrap_or(3000)
        } else {
            3000
        };
        
        // Check if this looks like the initial shield setting (current shields are 0)
        let current_shields = game_state.shields.unwrap_or(0);
        
        if current_shields == 0 {
            // Initial shield setting - use random value between 0-1000, but don't exceed available energy
            let max_initial_shields = std::cmp::min(1000, energy);
            let units = self.rng.gen_range(0..=max_initial_shields);
            return Ok(units.to_string());
        }
        
        // Subsequent shield adjustments - be more defensive
        // Use 30-70% of available energy for shields
        let min_shields = (energy as f32 * 0.3) as i32;
        let max_shields = (energy as f32 * 0.7) as i32;
        let units = self.rng.gen_range(min_shields..=max_shields);
        
        Ok(units.to_string())
    }
    
    /// Handle warp factor prompt
    fn handle_warp_factor(&mut self, game_state: &GameState) -> Result<String> {
        if let Some(last_output) = game_state.last_output.last() {
            if let Some((_min, max)) = parse_warp_factor_range(last_output) {
                if max <= 0.2 {
                    // Damaged warp engines
                    let factor = self.rng.gen::<f32>() / 4.0;
                    return Ok(format!("{:.2}", factor));
                }
            }
        }
        
        // Default range - warp factors are typically 0.1 to 8.0
        let factor = self.rng.gen_range(0.1..8.0);
        Ok(format!("{:.1}", factor))
    }
    
    /// Handle coordinates prompt
    fn handle_coordinates(&mut self, _game_state: &GameState) -> Result<String> {
        let x = self.rng.gen_range(1..9);
        let y = self.rng.gen_range(1..9);
        Ok(format!("{},{}", x, y))
    }
    
    /// Handle phaser units prompt
    fn handle_phaser_units(&mut self, _game_state: &GameState) -> Result<String> {
        let units = self.rng.gen_range(1..500);
        Ok(units.to_string())
    }
    
    /// Handle AYE prompt
    fn handle_aye_prompt(&mut self, _game_state: &GameState) -> Result<String> {
        Ok("quit".to_string())
    }
    
    /// Handle repair authorization prompt
    fn handle_repair_prompt(&mut self, _game_state: &GameState) -> Result<String> {
        if self.rng.gen_bool(0.5) {
            Ok("Y".to_string())
        } else {
            Ok("N".to_string())
        }
    }
    
    /// Handle energy available prompt
    fn handle_energy_prompt(&mut self, energy_value: i32) -> Result<String> {
        let units = self.rng.gen_range(1..=energy_value);
        Ok(units.to_string())
    }
}

impl Strategy for RandomStrategy {
    fn get_command(&mut self, game_state: &GameState) -> Result<String> {
        let prompt = game_state.get_current_prompt().unwrap_or("").trim();
        
        log::debug!("Random strategy handling prompt: '{}'", prompt);
        
        // If the prompt is just "?", look at the full context to understand what's being asked
        let effective_prompt = if prompt == "?" {
            // Look at the last few lines to find the context
            let recent_lines = game_state.last_output.iter()
                .rev()
                .take(3)
                .collect::<Vec<_>>();
            
            let mut context_prompt = prompt;
            for line in recent_lines {
                if line.contains("WARP FACTOR") {
                    context_prompt = "WARP FACTOR";
                    break;
                }
                if line.contains("COURSE (0-9)") {
                    context_prompt = "COURSE (0-9)";
                    break;
                }
                if line.contains("PHOTON TORPEDO COURSE") {
                    context_prompt = "PHOTON TORPEDO COURSE";
                    break;
                }
                if line.contains("NUMBER OF UNITS TO SHIELDS") {
                    context_prompt = "NUMBER OF UNITS TO SHIELDS";
                    break;
                }
                if line.contains("NUMBER OF UNITS TO FIRE") {
                    context_prompt = "NUMBER OF UNITS TO FIRE";
                    break;
                }
                if line.contains("INITIAL COORDINATES (X,Y)") {
                    context_prompt = "INITIAL COORDINATES (X,Y)";
                    break;
                }
                if line.contains("FINAL COORDINATES (X,Y)") {
                    context_prompt = "FINAL COORDINATES (X,Y)";
                    break;
                }
                if line.contains("COMPUTER ACTIVE AND AWAITING COMMAND") {
                    context_prompt = "COMPUTER ACTIVE AND AWAITING COMMAND";
                    break;
                }
            }
            context_prompt
        } else {
            prompt
        };
        
        log::debug!("Effective prompt after context detection: '{}'", effective_prompt);
        
        match effective_prompt {
            // Main command prompt
            "COMMAND" | "COMMAND?" => self.handle_command_prompt(game_state),
            "ENTER ONE OF THE FOLLOWING:" => {
                // This is just a menu header, not a command prompt
                // The game will show the menu and then prompt for COMMAND?
                Ok("".to_string())
            },
            "PLEASE ENTER" => {
                // This is just a print statement before the actual INPUT prompt
                // The game will show this and then prompt for coordinates
                Ok("".to_string())
            },
            
            // Navigation prompts
            p if p.contains("COURSE (0-9)") => self.handle_course_prompt(game_state),
            p if p.contains("WARP FACTOR") => self.handle_warp_factor(game_state),
            
            // Weapon prompts
            p if p.contains("PHOTON TORPEDO COURSE") => self.handle_torpedo_course(game_state),
            p if p.contains("NUMBER OF UNITS TO FIRE") => self.handle_phaser_units(game_state),
            p if p.contains("PHASERS LOCKED ON TARGET") && p.contains("ENERGY AVAILABLE") => {
                // Handle phaser targeting prompt like "PHASERS LOCKED ON TARGET; ENERGY AVAILABLE = 3000 UNITS"
                if let Some(energy) = parse_energy_available(p) {
                    self.handle_energy_prompt(energy)
                } else {
                    Err(anyhow::anyhow!("Could not parse energy value from: {}", p))
                }
            }
            
            // Shield and energy prompts
            p if p.contains("NUMBER OF UNITS TO SHIELDS") => self.handle_shield_units(game_state),
            p if p.starts_with("ENERGY AVAILABLE = ") => {
                if let Some(energy) = parse_energy_available(p) {
                    self.handle_energy_prompt(energy)
                } else {
                    Err(anyhow::anyhow!("Could not parse energy value from: {}", p))
                }
            }
            
            // Computer prompts
            p if p.contains("COMPUTER ACTIVE AND AWAITING COMMAND") => self.handle_computer_command(game_state),
            p if p.contains("INITIAL COORDINATES (X,Y)") => self.handle_coordinates(game_state),
            p if p.contains("FINAL COORDINATES (X,Y)") => self.handle_coordinates(game_state),
            
            // Repair and maintenance prompts
            p if p.contains("WILL YOU AUTHORIZE THE REPAIR ORDER") => self.handle_repair_prompt(game_state),
            p if p.contains("SHIELD CONTROL INOPERABLE") => {
                // Pick a different command since shields are broken
                self.handle_command_prompt(game_state)
            }
            
            // Mission control prompts
            p if p.contains("LET HIM STEP FORWARD AND ENTER 'AYE'") => self.handle_aye_prompt(game_state),
            
            // Status messages and reports that just need Enter to continue
            p if p.contains("LT. UHURA REPORTS MESSAGE") => {
                Ok("".to_string())
            }
            p if p.contains("SHIELDS NOW AT") && p.contains("UNITS PER YOUR COMMAND") => {
                // Status message after shield changes - just continue
                Ok("".to_string())
            }
            p if p.contains("DEFLECTOR CONTROL ROOM REPORT") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("DAMAGE CONTROL REPORT") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("ENGINEERING REPORTS") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("CHIEF ENGINEER SCOTT REPORTS") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("STARBASE SHIELDS PROTECT") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("SENSORS SHOW NO DAMAGE") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("UNIT HIT ON") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("KLINGON DESTROYED") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("TORPEDO TRACK") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("STAR AT") && p.contains("ABSORBED TORPEDO") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("STARBASE DESTROYED") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("TORPEDO MISSED") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("SHIELDS UNCHANGED") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("CONDITION RED") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("WARP ENGINES SHUT DOWN") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("PERMISSION TO ATTEMPT CROSSING") => {
                // Status message - just continue
                Ok("".to_string())
            }
            p if p.contains("NOW ENTERING") && p.contains("QUADRANT") => {
                // Status message when entering new quadrant - just continue
                Ok("".to_string())
            }
            
            // Help menu lines - these are just informational, not prompts
            p if p.contains("NAV  (TO SET COURSE)") => {
                Ok("".to_string())
            }
            p if p.contains("SRS  (FOR SHORT RANGE SENSOR SCAN)") => {
                Ok("".to_string())
            }
            p if p.contains("LRS  (FOR LONG RANGE SENSOR SCAN)") => {
                Ok("".to_string())
            }
            p if p.contains("PHA  (TO FIRE PHASERS)") => {
                Ok("".to_string())
            }
            p if p.contains("TOR  (TO FIRE PHOTON TORPEDOES)") => {
                Ok("".to_string())
            }
            p if p.contains("SHE  (TO RAISE OR LOWER SHIELDS)") => {
                Ok("".to_string())
            }
            p if p.contains("DAM  (FOR DAMAGE CONTROL REPORTS)") => {
                Ok("".to_string())
            }
            p if p.contains("COM  (TO CALL ON LIBRARY-COMPUTER)") => {
                Ok("".to_string())
            }
            p if p.contains("XXX  (TO RESIGN YOUR COMMAND)") => {
                Ok("".to_string())
            }
            
            // Header lines that precede the actual prompts - wait for real prompt
            p if p.contains("PLEASE ENTER") => {
                Ok("".to_string())
            }
            
            // Generic "?" prompt - couldn't determine context, just send Enter
            "?" => {
                log::warn!("Generic '?' prompt with no detectable context, sending empty response");
                Ok("".to_string())
            }
            
            _ => {
                log::warn!("Unknown prompt in random strategy: '{}'", prompt);
                Err(anyhow::anyhow!("Unknown prompt: '{}'", prompt))
            }
        }
    }
    
    fn reset(&mut self) {
        // Reset first_turn flag for new game
        self.first_turn = true;
    }
    
    fn name(&self) -> &'static str {
        "Random"
    }
}

impl Default for RandomStrategy {
    fn default() -> Self {
        Self::new()
    }
} 