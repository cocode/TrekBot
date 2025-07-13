use anyhow::Result;
use tokio::process::Child;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};

pub mod basicrs;
pub mod trekbasic;
pub mod trekbasicj;

/// Trait for communicating with different BASIC interpreters
#[async_trait::async_trait]
pub trait Interpreter {
    /// Launch the interpreter with the given BASIC program
    async fn launch(&mut self, program_path: &str) -> Result<()>;
    
    /// Send a command to the interpreter
    async fn send_command(&mut self, command: &str) -> Result<()>;
    
    /// Read the next line of output from the interpreter
    async fn read_line(&mut self) -> Result<Option<String>>;
    
    /// Read all available output until a prompt is detected
    async fn read_until_prompt(&mut self) -> Result<Vec<String>>;
    
    /// Check if the interpreter process is still running
    fn is_running(&mut self) -> bool;
    
    /// Terminate the interpreter process
    async fn terminate(&mut self) -> Result<()>;
}

/// Base structure for subprocess-based interpreters
pub struct SubprocessInterpreter {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
}

impl SubprocessInterpreter {
    pub fn new() -> Self {
        Self {
            process: None,
            stdin: None,
            stdout: None,
        }
    }
    
    pub async fn spawn_process(&mut self, command: &str, args: &[&str]) -> Result<()> {
        use tokio::process::Command;
        
        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()?;
        
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        
        self.process = Some(child);
        self.stdin = Some(stdin);
        self.stdout = Some(stdout);
        
        Ok(())
    }
    
    pub async fn write_line(&mut self, line: &str) -> Result<()> {
        if let Some(stdin) = &mut self.stdin {
            match stdin.write_all(line.as_bytes()).await {
                Ok(_) => {
                    match stdin.write_all(b"\n").await {
                        Ok(_) => {
                            match stdin.flush().await {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    log::error!("Failed to flush stdin: {}", e);
                                    // Check if the process has exited
                                    if !self.is_running_impl() {
                                        log::error!("Process has already exited, cannot send more commands");
                                    }
                                    Err(e.into())
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to write newline to stdin: {}", e);
                            if !self.is_running_impl() {
                                log::error!("Process has already exited, cannot send more commands");
                            }
                            Err(e.into())
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to write command '{}' to stdin: {}", line, e);
                    if !self.is_running_impl() {
                        log::error!("Process has already exited, cannot send more commands");
                    }
                    Err(e.into())
                }
            }
        } else {
            log::error!("No stdin available for writing");
            Ok(())
        }
    }
    
    pub async fn read_line_impl(&mut self) -> Result<Option<String>> {
        if let Some(stdout) = &mut self.stdout {
            let mut buffer = String::new();
            let mut byte_buffer = [0u8; 1];
            
            loop {
                match stdout.read(&mut byte_buffer).await {
                    Ok(0) => {
                        // EOF - process has likely terminated
                        log::debug!("EOF reached while reading from process");
                        if !self.is_running_impl() {
                            log::warn!("Process has terminated while reading output");
                        }
                        if buffer.is_empty() {
                            return Ok(None);
                        } else {
                            return Ok(Some(buffer));
                        }
                    }
                    Ok(_) => {
                        let ch = byte_buffer[0] as char;
                        
                        // Check for newline - complete line
                        if ch == '\n' {
                            // Remove trailing \r if present
                            if buffer.ends_with('\r') {
                                buffer.pop();
                            }
                            return Ok(Some(buffer));
                        }
                        
                        // Check for prompt character without newline
                        if ch == '?' {
                            buffer.push(ch);
                            return Ok(Some(buffer));
                        }
                        
                        // Regular character
                        buffer.push(ch);
                    }
                    Err(e) => {
                        log::error!("Error reading from process stdout: {}", e);
                        if !self.is_running_impl() {
                            log::error!("Process has terminated, cannot read more output");
                        }
                        return Err(e.into());
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
    
    pub fn is_running_impl(&mut self) -> bool {
        if let Some(process) = &mut self.process {
            // For tokio::process::Child, we can use try_wait to check if the process has exited
            // This is non-blocking and returns None if still running
            match process.try_wait() {
                Ok(Some(exit_status)) => {
                    // Process has exited - log the exit code
                    log::warn!("BasicRS process has exited with status: {:?}", exit_status);
                    false
                }
                Ok(None) => true,     // Process is still running
                Err(e) => {
                    log::error!("Error checking process status: {}", e);
                    false      // Error checking status, assume not running
                }
            }
        } else {
            false
        }
    }
    
    pub async fn terminate_impl(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            // First try to send a quit command to allow graceful shutdown
            if let Err(e) = self.write_line("XXX").await {
                log::debug!("Failed to send quit command: {}", e);
            }
            
            // Wait a bit for graceful shutdown
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            // Check if process has exited gracefully
            if let Ok(Some(exit_status)) = process.try_wait() {
                log::debug!("Process exited gracefully with status: {:?}", exit_status);
            } else {
                // Process hasn't exited, kill it
                log::debug!("Process didn't exit gracefully, killing it");
                process.kill().await?;
                let _ = process.wait().await?;
            }
        }
        self.stdin = None;
        self.stdout = None;
        Ok(())
    }
}

/// Common prompts that indicate the game is waiting for input
pub const GAME_PROMPTS: &[&str] = &[
    "COMMAND?",  // Changed from "COMMAND" to be more specific
    "COURSE (0-9)",
    "WARP FACTOR",
    "PHOTON TORPEDO COURSE (1-9)",
    "NUMBER OF UNITS TO SHIELDS",
    "NUMBER OF UNITS TO FIRE",
    "COMPUTER ACTIVE AND AWAITING COMMAND",
    "INITIAL COORDINATES (X,Y)",
    "FINAL COORDINATES (X,Y)",
    "LET HIM STEP FORWARD AND ENTER 'AYE'",
    "WILL YOU AUTHORIZE THE REPAIR ORDER (Y/N)",
    "ENERGY AVAILABLE =",
    "HIT ANY KEY", // For initial startup
    "PRESS ANY KEY", // Alternative wording
    "WHEN READY", // From the original game
    "?", // Generic prompt indicator
];

/// Check if a line contains a game prompt
pub fn is_game_prompt(line: &str) -> bool {
    let line = line.trim();
    
    // Skip help menu lines - these are informational, not prompts
    if line.contains("NAV  (TO SET COURSE)") ||
       line.contains("SRS  (FOR SHORT RANGE SENSOR SCAN)") ||
       line.contains("LRS  (FOR LONG RANGE SENSOR SCAN)") ||
       line.contains("PHA  (TO FIRE PHASERS)") ||
       line.contains("TOR  (TO FIRE PHOTON TORPEDOES)") ||
       line.contains("SHE  (TO RAISE OR LOWER SHIELDS)") ||
       line.contains("DAM  (FOR DAMAGE CONTROL REPORTS)") ||
       line.contains("COM  (TO CALL ON LIBRARY-COMPUTER)") ||
       line.contains("XXX  (TO RESIGN YOUR COMMAND)") {
        return false;
    }
    
    // Skip specific status messages that contain "ENTER" but are not prompts
    if line.contains("NOW ENTERING") && line.contains("QUADRANT") {
        return false;
    }
    
    // Skip "PLEASE ENTER" - it's just a print statement before the real INPUT prompts
    if line.contains("PLEASE ENTER") {
        return false;
    }
    
    // Skip computer command output headers
    if line.contains("FROM ENTERPRISE TO KLINGON BATTLE CRUSER") {
        return false;
    }
    
    // Skip other computer command direction headers
    if line.contains("FROM ENTERPRISE TO STARBASE") {
        return false;
    }
    
    // Skip damage status messages that contain "ENTERPRISE" 
    if line.contains("UNIT HIT ON ENTERPRISE") {
        return false;
    }
    
    // Skip game-over messages
    if line.contains("THE ENTERPRISE HAS BEEN DESTROYED") {
        return false;
    }
    
    // Skip phaser targeting status messages
    if line.contains("PHASERS LOCKED ON TARGET") {
        return false;
    }
    
    // Check for exact matches or contains
    for prompt in GAME_PROMPTS {
        if line.contains(prompt) || line.ends_with(prompt) {
            return true;
        }
    }
    
    // Check for input prompts that end with specific characters
    if line.ends_with('?') {
        return true;
    }
    
    // Check for lines that look like they're waiting for input
    if line.contains("INPUT") || line.contains("ENTER") {
        return true;
    }
    
    // Check for BASIC INPUT statement patterns
    if line.contains("INPUT") && (line.contains("COMMAND") || line.contains("COURSE") || line.contains("FACTOR")) {
        return true;
    }
    
    false
}

/// Check if we should send an initial response to start the game
pub fn needs_initial_response(line: &str) -> bool {
    let line = line.trim().to_uppercase();
    line.contains("HIT ANY KEY") 
        || line.contains("PRESS ANY KEY")
        || line.contains("WHEN READY")
        || (line.contains("COMMAND") && !line.contains("="))
        || line.contains("INPUT")
} 