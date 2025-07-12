use anyhow::Result;
use super::{Interpreter, SubprocessInterpreter, is_game_prompt};

/// TrekBasicJ (Java) interpreter implementation
pub struct TrekBasicJInterpreter {
    subprocess: SubprocessInterpreter,
    java_path: String,
    jar_path: String,
}

impl TrekBasicJInterpreter {
    pub fn new(java_path: Option<String>, jar_path: Option<String>) -> Self {
        let default_java = "java".to_string();
        let default_jar = "/path/to/trekbasicj.jar".to_string(); // TODO: Update when available
        
        Self {
            subprocess: SubprocessInterpreter::new(),
            java_path: java_path.unwrap_or(default_java),
            jar_path: jar_path.unwrap_or(default_jar),
        }
    }
}

#[async_trait::async_trait]
impl Interpreter for TrekBasicJInterpreter {
    async fn launch(&mut self, program_path: &str) -> Result<()> {
        log::info!("Launching TrekBasicJ interpreter with program: {}", program_path);
        
        // Launch the Java interpreter with the JAR file and program
        self.subprocess.spawn_process(&self.java_path, &["-jar", &self.jar_path, program_path]).await?;
        
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
        log::info!("Terminating TrekBasicJ interpreter");
        self.subprocess.terminate_impl().await
    }
} 