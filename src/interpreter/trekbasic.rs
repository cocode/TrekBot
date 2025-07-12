use anyhow::Result;
use super::{Interpreter, SubprocessInterpreter, is_game_prompt};

/// TrekBasic (Python) interpreter implementation
pub struct TrekBasicInterpreter {
    subprocess: SubprocessInterpreter,
    python_path: String,
    script_path: String,
}

impl TrekBasicInterpreter {
    pub fn new(python_path: Option<String>, script_path: Option<String>) -> Self {
        let default_python = "python3".to_string();
        let default_script = "/Users/tomhill/PycharmProjects/TrekBasic/basic.py".to_string();
        
        Self {
            subprocess: SubprocessInterpreter::new(),
            python_path: python_path.unwrap_or(default_python),
            script_path: script_path.unwrap_or(default_script),
        }
    }
}

#[async_trait::async_trait]
impl Interpreter for TrekBasicInterpreter {
    async fn launch(&mut self, program_path: &str) -> Result<()> {
        log::info!("Launching TrekBasic interpreter with program: {}", program_path);
        
        // Launch the Python interpreter with the basic.py script and program
        self.subprocess.spawn_process(&self.python_path, &[&self.script_path, program_path]).await?;
        
        // Read initial output until we get a prompt
        let _initial_output = self.read_until_prompt().await?;
        
        Ok(())
    }
    
    async fn send_command(&mut self, command: &str) -> Result<()> {
        log::debug!("Sending command: {}", command);
        self.subprocess.write_line(command).await
    }
    
    async fn read_line(&mut self) -> Result<Option<String>> {
        self.subprocess.read_line_impl().await
    }
    
    async fn read_until_prompt(&mut self) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        
        while let Some(line) = self.read_line().await? {
            lines.push(line.clone());
            log::debug!("Read line: {}", line);
            
            if is_game_prompt(&line) {
                log::debug!("Found game prompt: {}", line);
                break;
            }
        }
        
        Ok(lines)
    }
    
    fn is_running(&mut self) -> bool {
        self.subprocess.is_running_impl()
    }
    
    async fn terminate(&mut self) -> Result<()> {
        log::info!("Terminating TrekBasic interpreter");
        self.subprocess.terminate_impl().await
    }
} 