use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Current game state extracted from interpreter output
#[derive(Debug, Clone)]
pub struct GameState {
    pub current_quadrant: Option<(i32, i32)>,
    pub current_sector: Option<(i32, i32)>,
    pub energy: Option<i32>,
    pub shields: Option<i32>,
    pub torpedoes: Option<i32>,
    pub klingons_remaining: Option<i32>,
    pub time_remaining: Option<i32>,
    pub starbases: Option<i32>,
    pub stardate: Option<i32>,
    pub last_prompt: Option<String>,
    pub last_output: Vec<String>,
    pub condition: Option<String>,
    pub damage_report: HashMap<String, f32>,
    pub galaxy_map: Option<Vec<Vec<String>>>,
    pub sector_map: Option<Vec<Vec<String>>>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            current_quadrant: None,
            current_sector: None,
            energy: None,
            shields: None,
            torpedoes: None,
            klingons_remaining: None,
            time_remaining: None,
            starbases: None,
            stardate: None,
            last_prompt: None,
            last_output: Vec::new(),
            condition: None,
            damage_report: HashMap::new(),
            galaxy_map: None,
            sector_map: None,
        }
    }
    
    /// Update the game state with new output from the interpreter
    pub fn update(&mut self, output: &[String]) -> Result<()> {
        self.last_output = output.to_vec();
        
        // Find the last prompt
        if let Some(last_line) = output.last() {
            if crate::interpreter::is_game_prompt(last_line) {
                self.last_prompt = Some(last_line.clone());
            }
        }
        
        // Parse various game state information from output
        for line in output {
            self.parse_energy(line)?;
            self.parse_shields(line)?;
            self.parse_torpedoes(line)?;
            self.parse_klingons(line)?;
            self.parse_time(line)?;
            self.parse_condition(line)?;
            self.parse_quadrant(line)?;
            self.parse_sector(line)?;
            self.parse_stardate(line)?;
            self.parse_damage_report(line)?;
        }
        
        Ok(())
    }
    
    fn parse_energy(&mut self, line: &str) -> Result<()> {
        let energy_regex = Regex::new(r"(?:TOTAL\s+)?ENERGY\s*[=:]?\s*(\d+)")?;
        if let Some(caps) = energy_regex.captures(line) {
            if let Some(energy_str) = caps.get(1) {
                self.energy = energy_str.as_str().parse().ok();
            }
        }
        
        // Also match energy available prompts
        let energy_available_regex = Regex::new(r"ENERGY AVAILABLE\s*=\s*(\d+)")?;
        if let Some(caps) = energy_available_regex.captures(line) {
            if let Some(energy_str) = caps.get(1) {
                self.energy = energy_str.as_str().parse().ok();
            }
        }
        Ok(())
    }
    
    fn parse_shields(&mut self, line: &str) -> Result<()> {
        // Match the main status display format
        let shields_regex = Regex::new(r"SHIELDS\s*[=:]?\s*(\d+)")?;
        if let Some(caps) = shields_regex.captures(line) {
            if let Some(shields_str) = caps.get(1) {
                self.shields = shields_str.as_str().parse().ok();
            }
        }
        
        // Also match shield status messages
        let shield_status_regex = Regex::new(r"SHIELDS NOW AT\s*(\d+)\s*UNITS")?;
        if let Some(caps) = shield_status_regex.captures(line) {
            if let Some(shields_str) = caps.get(1) {
                self.shields = shields_str.as_str().parse().ok();
            }
        }
        Ok(())
    }
    
    fn parse_torpedoes(&mut self, line: &str) -> Result<()> {
        let torpedoes_regex = Regex::new(r"(?:PHOTON\s+)?TORPEDOES\s*[=:]?\s*(\d+)")?;
        if let Some(caps) = torpedoes_regex.captures(line) {
            if let Some(torpedoes_str) = caps.get(1) {
                self.torpedoes = torpedoes_str.as_str().parse().ok();
            }
        }
        Ok(())
    }
    
    fn parse_klingons(&mut self, line: &str) -> Result<()> {
        // Try "KLINGONS REMAINING 13" format first
        let remaining_regex = Regex::new(r"KLINGONS?\s+REMAINING\s+(\d+)")?;
        if let Some(caps) = remaining_regex.captures(line) {
            if let Some(klingons_str) = caps.get(1) {
                self.klingons_remaining = klingons_str.as_str().parse().ok();
                return Ok(());
            }
        }
        
        // Try "13 KLINGON" format
        let count_regex = Regex::new(r"(\d+)\s*KLINGON")?;
        if let Some(caps) = count_regex.captures(line) {
            if let Some(klingons_str) = caps.get(1) {
                self.klingons_remaining = klingons_str.as_str().parse().ok();
            }
        }
        Ok(())
    }
    
    fn parse_time(&mut self, line: &str) -> Result<()> {
        let time_regex = Regex::new(r"TIME\s*[=:]\s*(\d+)")?;
        if let Some(caps) = time_regex.captures(line) {
            if let Some(time_str) = caps.get(1) {
                self.time_remaining = time_str.as_str().parse().ok();
            }
        }
        Ok(())
    }
    
    fn parse_condition(&mut self, line: &str) -> Result<()> {
        if line.contains("CONDITION") && line.contains("RED") {
            self.condition = Some("RED".to_string());
        } else if line.contains("CONDITION") && line.contains("GREEN") {
            self.condition = Some("GREEN".to_string());
        } else if line.contains("CONDITION") && line.contains("YELLOW") {
            self.condition = Some("YELLOW".to_string());
        }
        Ok(())
    }
    
    fn parse_quadrant(&mut self, line: &str) -> Result<()> {
        let quadrant_regex = Regex::new(r"QUADRANT\s*[=:]?\s*(\d+)\s*,\s*(\d+)")?;
        if let Some(caps) = quadrant_regex.captures(line) {
            if let (Some(q1), Some(q2)) = (caps.get(1), caps.get(2)) {
                let q1: i32 = q1.as_str().parse().unwrap_or(0);
                let q2: i32 = q2.as_str().parse().unwrap_or(0);
                self.current_quadrant = Some((q1, q2));
            }
        }
        Ok(())
    }
    
    fn parse_sector(&mut self, line: &str) -> Result<()> {
        let sector_regex = Regex::new(r"SECTOR\s*[=:]?\s*(\d+)\s*,\s*(\d+)")?;
        if let Some(caps) = sector_regex.captures(line) {
            if let (Some(s1), Some(s2)) = (caps.get(1), caps.get(2)) {
                let s1: i32 = s1.as_str().parse().unwrap_or(0);
                let s2: i32 = s2.as_str().parse().unwrap_or(0);
                self.current_sector = Some((s1, s2));
            }
        }
        Ok(())
    }
    
    fn parse_stardate(&mut self, line: &str) -> Result<()> {
        let stardate_regex = Regex::new(r"STARDATE\s*[=:]?\s*(\d+)")?;
        if let Some(caps) = stardate_regex.captures(line) {
            if let Some(stardate_str) = caps.get(1) {
                self.stardate = stardate_str.as_str().parse().ok();
            }
        }
        Ok(())
    }
    
    fn parse_damage_report(&mut self, line: &str) -> Result<()> {
        // Parse damage reports like "WARP ENGINES DAMAGED"
        let damage_regex = Regex::new(r"([A-Z\s]+)\s+(DAMAGED|INOPERABLE|REPAIR)")?;
        if let Some(caps) = damage_regex.captures(line) {
            if let (Some(system), Some(status)) = (caps.get(1), caps.get(2)) {
                let system_name = system.as_str().trim().to_string();
                let damage_value = match status.as_str() {
                    "DAMAGED" => -1.0,
                    "INOPERABLE" => -2.0,
                    "REPAIR" => 0.0,
                    _ => 0.0,
                };
                self.damage_report.insert(system_name, damage_value);
            }
        }
        Ok(())
    }
    
    /// Get the current prompt, if any
    pub fn get_current_prompt(&self) -> Option<&str> {
        self.last_prompt.as_deref()
    }
    
    /// Check if the game is in a combat situation
    pub fn is_in_combat(&self) -> bool {
        self.condition.as_deref() == Some("RED")
    }
    
    /// Check if shields are dangerously low
    pub fn are_shields_low(&self) -> bool {
        self.shields.map_or(false, |s| s < 200)
    }
    
    /// Check if a system is damaged
    pub fn is_system_damaged(&self, system: &str) -> bool {
        self.damage_report.get(system).map_or(false, |&damage| damage < 0.0)
    }
    
    /// Display current game state in a concise format
    pub fn display_status(&self) {
        let stardate = self.stardate.map_or("???".to_string(), |d| d.to_string());
        let klingons = self.klingons_remaining.map_or("?".to_string(), |k| k.to_string());
        let energy = self.energy.map_or("????".to_string(), |e| e.to_string());
        let shields = self.shields.map_or("????".to_string(), |s| s.to_string());
        let torpedoes = self.torpedoes.map_or("??".to_string(), |t| t.to_string());
        let condition = self.condition.as_deref().unwrap_or("?????");
        
        let quadrant = if let Some((q1, q2)) = self.current_quadrant {
            format!("({},{})", q1, q2)
        } else {
            "(??,??)".to_string()
        };
        
        let sector = if let Some((s1, s2)) = self.current_sector {
            format!("({},{})", s1, s2)
        } else {
            "(??,??)".to_string()
        };
        
        println!("📊 Turn Status: Stardate {} | Klingons {} | Energy {} | Shields {} | Torpedoes {} | {} | Q{} S{}", 
                 stardate, klingons, energy, shields, torpedoes, condition, quadrant, sector);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
} 