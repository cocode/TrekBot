use anyhow::Result;
use super::{Interpreter, SubprocessInterpreter, is_game_prompt};

/// BasicRS interpreter implementation
pub struct BasicRSInterpreter {
    subprocess: SubprocessInterpreter,
    basicrs_path: String,
    coverage_file: Option<String>,
    reset_coverage: bool,
}

impl BasicRSInterpreter {
    pub fn new(basicrs_path: Option<String>) -> Self {
        let default_path = "/Users/tomhill/RustroverProjects/BasicRS/target/debug/basic_rs".to_string();
        Self {
            subprocess: SubprocessInterpreter::new(),
            basicrs_path: basicrs_path.unwrap_or(default_path),
            coverage_file: None,
            reset_coverage: false,
        }
    }
    
    pub fn set_coverage_file(&mut self, coverage_file: Option<String>) {
        println!("🔍 Setting coverage file: {:?}", coverage_file);
        self.coverage_file = coverage_file;
    }
    
    pub fn set_reset_coverage(&mut self, reset: bool) {
        self.reset_coverage = reset;
    }
}

#[async_trait::async_trait]
impl Interpreter for BasicRSInterpreter {
    async fn launch(&mut self, program_path: &str) -> Result<()> {
        log::info!("Launching BasicRS interpreter with program: {}", program_path);
        
        // Build arguments for BasicRS
        let mut args = vec![program_path];
        
        // Add coverage arguments if specified
        let coverage_file = self.coverage_file.as_deref().unwrap_or("coverage.json");
        args.push("--coverage-file");
        args.push(coverage_file);
        println!("🔍 Coverage file set to: {}", coverage_file);
        println!("🔍 Full coverage path: {}", std::path::Path::new(coverage_file).canonicalize().unwrap_or_else(|_| coverage_file.into()).display());
        
        if self.reset_coverage {
            args.push("--reset-coverage");
            println!("🔍 Coverage reset enabled");
        }
        
        println!("🔍 BasicRS command: {} {:?}", self.basicrs_path, args);
        
        // Launch the BasicRS interpreter with the program and arguments
        self.subprocess.spawn_process(&self.basicrs_path, &args).await?;
        
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