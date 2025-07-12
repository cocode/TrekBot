use anyhow::Result;
use regex::Regex;

/// Parse energy available from output like "ENERGY AVAILABLE = 3000 NUMBER OF UNITS TO SHIELDS"
pub fn parse_energy_available(line: &str) -> Option<i32> {
    let regex = Regex::new(r"ENERGY\s+AVAILABLE\s*=\s*(\d+)").ok()?;
    regex.captures(line)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Parse warp factor range from output like "WARP FACTOR (0-8)?" or "WARP FACTOR (0-0.2)?"
pub fn parse_warp_factor_range(line: &str) -> Option<(f32, f32)> {
    let regex = Regex::new(r"WARP\s+FACTOR\s*\((\d+(?:\.\d+)?)-(\d+(?:\.\d+)?)\)").ok()?;
    regex.captures(line)
        .and_then(|caps| {
            let min = caps.get(1)?.as_str().parse().ok()?;
            let max = caps.get(2)?.as_str().parse().ok()?;
            Some((min, max))
        })
}

/// Parse quadrant name from output like "NOW ENTERING ANTARES QUADRANT..."
pub fn parse_quadrant_name(line: &str) -> Option<String> {
    let regex = Regex::new(r"(?:NOW ENTERING|LOCATED IN)\s+([A-Z\s]+)\s+QUADRANT").ok()?;
    regex.captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Parse course from navigation output
pub fn parse_course_command(line: &str) -> Option<f32> {
    let regex = Regex::new(r"COURSE\s*\((\d+(?:\.\d+)?)-(\d+(?:\.\d+)?)\)").ok()?;
    regex.captures(line)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Parse coordinates from output like "INITIAL COORDINATES (X,Y)" or "FINAL COORDINATES (X,Y)"
pub fn parse_coordinates_prompt(line: &str) -> bool {
    line.contains("COORDINATES (X,Y)")
}

/// Parse damage control report sections
pub fn parse_damage_control_report(lines: &[String]) -> Vec<(String, f32)> {
    let mut damage_reports = Vec::new();
    let system_regex = Regex::new(r"([A-Z\s]+)\s+([\d\.-]+)").unwrap();
    
    for line in lines {
        if line.contains("DAMAGE CONTROL REPORT") {
            continue;
        }
        
        if let Some(caps) = system_regex.captures(line) {
            if let (Some(system), Some(damage_str)) = (caps.get(1), caps.get(2)) {
                let system_name = system.as_str().trim().to_string();
                if let Ok(damage_value) = damage_str.as_str().parse::<f32>() {
                    damage_reports.push((system_name, damage_value));
                }
            }
        }
    }
    
    damage_reports
}

/// Parse short range sensor scan to extract sector map
pub fn parse_short_range_scan(lines: &[String]) -> Option<Vec<Vec<String>>> {
    let mut sector_map = Vec::new();
    let mut in_scan = false;
    
    for line in lines {
        if line.contains("SHORT RANGE SENSORS") {
            in_scan = true;
            continue;
        }
        
        if in_scan {
            // Look for grid lines (containing sector positions)
            if line.contains("<*>") || line.contains("+K+") || line.contains(">!<") || line.contains(" * ") {
                let mut row = Vec::new();
                // Split the line into 3-character sectors
                let mut chars = line.chars().collect::<Vec<_>>();
                
                // Pad to ensure we have complete sectors
                while chars.len() % 3 != 0 {
                    chars.push(' ');
                }
                
                for chunk in chars.chunks(3) {
                    let sector: String = chunk.iter().collect();
                    row.push(sector);
                }
                
                if !row.is_empty() {
                    sector_map.push(row);
                }
            } else if line.trim().is_empty() && !sector_map.is_empty() {
                // End of scan
                break;
            }
        }
    }
    
    if sector_map.is_empty() {
        None
    } else {
        Some(sector_map)
    }
}

/// Parse long range sensor scan to extract galaxy map
pub fn parse_long_range_scan(lines: &[String]) -> Option<Vec<Vec<String>>> {
    let mut galaxy_map = Vec::new();
    let mut in_scan = false;
    
    for line in lines {
        if line.contains("LONG RANGE SCAN") {
            in_scan = true;
            continue;
        }
        
        if in_scan {
            if line.contains(":") && line.contains("***") || line.len() > 10 {
                // Parse galaxy quadrant line
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let quadrant_data = parts[1].trim();
                    let quadrants: Vec<String> = quadrant_data
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                    if !quadrants.is_empty() {
                        galaxy_map.push(quadrants);
                    }
                }
            } else if line.contains("---") {
                // Skip separator lines
                continue;
            } else if line.trim().is_empty() && !galaxy_map.is_empty() {
                // End of scan
                break;
            }
        }
    }
    
    if galaxy_map.is_empty() {
        None
    } else {
        Some(galaxy_map)
    }
}

/// Parse computer command output for galactic record
pub fn parse_galactic_record(lines: &[String]) -> Option<Vec<(i32, i32, String)>> {
    let mut records = Vec::new();
    let record_regex = Regex::new(r"(\d+),(\d+)\s+(.+)").unwrap();
    
    for line in lines {
        if line.contains("GALACTIC RECORD") {
            continue;
        }
        
        if let Some(caps) = record_regex.captures(line) {
            if let (Some(q1), Some(q2), Some(data)) = (caps.get(1), caps.get(2), caps.get(3)) {
                let q1: i32 = q1.as_str().parse().unwrap_or(0);
                let q2: i32 = q2.as_str().parse().unwrap_or(0);
                let data = data.as_str().trim().to_string();
                records.push((q1, q2, data));
            }
        }
    }
    
    if records.is_empty() {
        None
    } else {
        Some(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_energy_available() {
        assert_eq!(parse_energy_available("ENERGY AVAILABLE = 3000 NUMBER OF UNITS TO SHIELDS"), Some(3000));
        assert_eq!(parse_energy_available("ENERGY AVAILABLE = 1500"), Some(1500));
        assert_eq!(parse_energy_available("NO ENERGY INFO"), None);
    }
    
    #[test]
    fn test_parse_warp_factor_range() {
        assert_eq!(parse_warp_factor_range("WARP FACTOR (0-8)?"), Some((0.0, 8.0)));
        assert_eq!(parse_warp_factor_range("WARP FACTOR (0-0.2)?"), Some((0.0, 0.2)));
        assert_eq!(parse_warp_factor_range("INVALID"), None);
    }
    
    #[test]
    fn test_parse_quadrant_name() {
        assert_eq!(parse_quadrant_name("NOW ENTERING ANTARES QUADRANT..."), Some("ANTARES".to_string()));
        assert_eq!(parse_quadrant_name("LOCATED IN RIGEL QUADRANT"), Some("RIGEL".to_string()));
        assert_eq!(parse_quadrant_name("NO QUADRANT INFO"), None);
    }
} 