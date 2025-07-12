use anyhow::Result;
use super::{Interpreter, SubprocessInterpreter, is_game_prompt};

/// BasicRS interpreter implementation
pub struct BasicRSInterpreter {
    subprocess: SubprocessInterpreter,
    basicrs_path: String,
}

impl BasicRSInterpreter {
    pub fn new(basicrs_path: Option<String>) -> Self {
        let default_path = "/Users/tomhill/RustroverProjects/BasicRS/target/debug/basic_rs".to_string();
        Self {
            subprocess: SubprocessInterpreter::new(),
            basicrs_path: basicrs_path.unwrap_or(default_path),
        }
    }
}

#[async_trait::async_trait]
impl Interpreter for BasicRSInterpreter {
    async fn launch(&mut self, program_path: &str) -> Result<()> {
        log::info!("Launching BasicRS interpreter with program: {}", program_path);
        
        // Launch the BasicRS interpreter with the program
        self.subprocess.spawn_process(&self.basicrs_path, &[program_path]).await?;
        
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
        use tokio::time::{timeout, Duration};
        
        let mut lines = Vec::new();
        
        loop {
            match timeout(Duration::from_secs(2), self.read_line()).await {
                Ok(Ok(Some(line))) => {
                    lines.push(line.clone());
                    log::debug!("Read line: {}", line);
                    
                    if is_game_prompt(&line) {
                        log::debug!("Found game prompt: {}", line);
                        break;
                    }
                }
                Ok(Ok(None)) => {
                    log::debug!("End of output reached");
                    break;
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    log::debug!("Timeout reading line, stopping");
                    break;
                }
            }
        }
        
        Ok(lines)
    }
    
    fn is_running(&mut self) -> bool {
        self.subprocess.is_running_impl()
    }
    
    async fn terminate(&mut self) -> Result<()> {
        log::info!("Terminating BasicRS interpreter");
        self.subprocess.terminate_impl().await
    }
} 