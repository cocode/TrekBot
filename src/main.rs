mod game;
mod interpreter;
mod player;
mod strategy;

use anyhow::Result;
use clap::{Parser, Subcommand};
use interpreter::{
    basicrs::BasicRSInterpreter, 
    trekbasic::TrekBasicInterpreter, 
    trekbasicj::TrekBasicJInterpreter,
    Interpreter
};
use player::{GameStats, Player};
use strategy::{CheatStrategy, RandomStrategy};
use std::fs;
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Play a single game
    Play {
        /// Path to the Super Star Trek BASIC program
        #[arg(short, long)]
        program: String,
        
        /// Interpreter to use
        #[arg(short, long, default_value = "basic-rs")]
        interpreter: InterpreterType,
        
        /// Strategy to use
        #[arg(short, long, default_value = "random")]
        strategy: StrategyType,
        
        /// Display game output
        #[arg(short, long, default_value_t = false)]
        display: bool,
        
        /// Maximum number of turns
        #[arg(short, long, default_value_t = 100)]
        max_turns: usize,
        
        /// Path to BasicRS executable
        #[arg(long)]
        basicrs_path: Option<String>,
        
        /// Path to Python executable
        #[arg(long)]
        python_path: Option<String>,
        
        /// Path to TrekBasic script
        #[arg(long)]
        trekbasic_path: Option<String>,
        
        /// Path to Java executable
        #[arg(long)]
        java_path: Option<String>,
        
        /// Path to TrekBasicJ JAR
        #[arg(long)]
        trekbasicj_path: Option<String>,
    },
    
    /// Run multiple games and collect statistics
    Benchmark {
        /// Path to the Super Star Trek BASIC program
        #[arg(short, long)]
        program: String,
        
        /// Interpreter to use
        #[arg(short, long, default_value = "basic-rs")]
        interpreter: InterpreterType,
        
        /// Strategy to use
        #[arg(short, long, default_value = "random")]
        strategy: StrategyType,
        
        /// Number of games to play
        #[arg(short, long, default_value_t = 10)]
        games: usize,
        
        /// Display game output
        #[arg(short, long, default_value_t = false)]
        display: bool,
        
        /// Maximum number of turns per game
        #[arg(short, long, default_value_t = 100)]
        max_turns: usize,
        
        /// Path to BasicRS executable
        #[arg(long)]
        basicrs_path: Option<String>,
        
        /// Path to Python executable
        #[arg(long)]
        python_path: Option<String>,
        
        /// Path to TrekBasic script
        #[arg(long)]
        trekbasic_path: Option<String>,
        
        /// Path to Java executable
        #[arg(long)]
        java_path: Option<String>,
        
        /// Path to TrekBasicJ JAR
        #[arg(long)]
        trekbasicj_path: Option<String>,
        
        /// Enable coverage tracking and save to file
        #[arg(long)]
        coverage_file: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum InterpreterType {
    #[value(name = "basic-rs")]
    BasicRS,
    #[value(name = "trek-basic")]
    TrekBasic,
    #[value(name = "trek-basic-j")]
    TrekBasicJ,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum StrategyType {
    Random,
    Cheat,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Play {
            program,
            interpreter,
            strategy,
            display,
            max_turns,
            basicrs_path,
            python_path,
            trekbasic_path,
            java_path,
            trekbasicj_path,
        } => {
            play_single_game(
                program,
                interpreter,
                strategy,
                *display,
                *max_turns,
                basicrs_path,
                python_path,
                trekbasic_path,
                java_path,
                trekbasicj_path,
            )
            .await?;
        }
        Commands::Benchmark {
            program,
            interpreter,
            strategy,
            games,
            display,
            max_turns,
            basicrs_path,
            python_path,
            trekbasic_path,
            java_path,
            trekbasicj_path,
            coverage_file,
        } => {
            run_benchmark(
                program,
                interpreter,
                strategy,
                *games,
                *display,
                *max_turns,
                basicrs_path,
                python_path,
                trekbasic_path,
                java_path,
                trekbasicj_path,
                coverage_file,
            )
            .await?;
        }
    }
    
    Ok(())
}

async fn play_single_game(
    program: &str,
    interpreter_type: &InterpreterType,
    strategy_type: &StrategyType,
    display: bool,
    max_turns: usize,
    basicrs_path: &Option<String>,
    python_path: &Option<String>,
    trekbasic_path: &Option<String>,
    java_path: &Option<String>,
    trekbasicj_path: &Option<String>,
) -> Result<()> {
    let start_time = Instant::now();
    match (interpreter_type, strategy_type) {
        (InterpreterType::BasicRS, StrategyType::Random) => {
            let interpreter = BasicRSInterpreter::new(basicrs_path.clone());
            let strategy = RandomStrategy::new();
            let mut player = Player::new(interpreter, strategy, display);
            player.set_max_turns(max_turns);
            
            let result = player.play_game(program).await?;
            println!("Game Result: {} ({})", result.description(), player.get_turn_count());
        }
        (InterpreterType::BasicRS, StrategyType::Cheat) => {
            let interpreter = BasicRSInterpreter::new(basicrs_path.clone());
            let strategy = CheatStrategy::new();
            let mut player = Player::new(interpreter, strategy, display);
            player.set_max_turns(max_turns);
            
            let result = player.play_game(program).await?;
            println!("Game Result: {} ({})", result.description(), player.get_turn_count());
        }
        (InterpreterType::TrekBasic, StrategyType::Random) => {
            let interpreter = TrekBasicInterpreter::new(python_path.clone(), trekbasic_path.clone());
            let strategy = RandomStrategy::new();
            let mut player = Player::new(interpreter, strategy, display);
            player.set_max_turns(max_turns);
            
            let result = player.play_game(program).await?;
            println!("Game Result: {} ({})", result.description(), player.get_turn_count());
        }
        (InterpreterType::TrekBasic, StrategyType::Cheat) => {
            let interpreter = TrekBasicInterpreter::new(python_path.clone(), trekbasic_path.clone());
            let strategy = CheatStrategy::new();
            let mut player = Player::new(interpreter, strategy, display);
            player.set_max_turns(max_turns);
            
            let result = player.play_game(program).await?;
            println!("Game Result: {} ({})", result.description(), player.get_turn_count());
        }
        (InterpreterType::TrekBasicJ, StrategyType::Random) => {
            let interpreter = TrekBasicJInterpreter::new(java_path.clone(), trekbasicj_path.clone());
            let strategy = RandomStrategy::new();
            let mut player = Player::new(interpreter, strategy, display);
            player.set_max_turns(max_turns);
            
            let result = player.play_game(program).await?;
            println!("Game Result: {} ({})", result.description(), player.get_turn_count());
        }
        (InterpreterType::TrekBasicJ, StrategyType::Cheat) => {
            let interpreter = TrekBasicJInterpreter::new(java_path.clone(), trekbasicj_path.clone());
            let strategy = CheatStrategy::new();
            let mut player = Player::new(interpreter, strategy, display);
            player.set_max_turns(max_turns);
            
            let result = player.play_game(program).await?;
            println!("Game Result: {} ({})", result.description(), player.get_turn_count());
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("Total elapsed time: {:.2} seconds", elapsed.as_secs_f64());
    
    Ok(())
}

async fn run_benchmark(
    program: &str,
    interpreter_type: &InterpreterType,
    strategy_type: &StrategyType,
    games: usize,
    display: bool,
    max_turns: usize,
    basicrs_path: &Option<String>,
    python_path: &Option<String>,
    trekbasic_path: &Option<String>,
    java_path: &Option<String>,
    trekbasicj_path: &Option<String>,
    coverage_file: &Option<String>,
) -> Result<()> {
    let mut stats = GameStats::new();
    
    // Coverage will be handled by BasicRS itself
    
    println!("Running {} games with {} interpreter and {} strategy...", 
             games, 
             format!("{:?}", interpreter_type).to_lowercase(), 
             format!("{:?}", strategy_type).to_lowercase());
    
    for i in 0..games {
        println!("Game {}/{}", i + 1, games);
        
        let result = match (interpreter_type, strategy_type) {
            (InterpreterType::BasicRS, StrategyType::Random) => {
                let mut interpreter = BasicRSInterpreter::new(basicrs_path.clone());
                
                // Set coverage options if requested
                if let Some(ref coverage_file) = coverage_file {
                    interpreter.set_coverage_file(Some(coverage_file.clone()));
                    interpreter.set_reset_coverage(i == 0); // Reset only on first game
                }
                
                let strategy = RandomStrategy::new();
                let mut player = Player::new(interpreter, strategy, display);
                player.set_max_turns(max_turns);
                
                let result = player.play_game(program).await?;
                let turns = player.get_turn_count();
                stats.add_game(result.clone(), turns);
                result
            }
            (InterpreterType::BasicRS, StrategyType::Cheat) => {
                let mut interpreter = BasicRSInterpreter::new(basicrs_path.clone());
                
                // Set coverage options if requested
                if let Some(ref coverage_file) = coverage_file {
                    interpreter.set_coverage_file(Some(coverage_file.clone()));
                    interpreter.set_reset_coverage(i == 0); // Reset only on first game
                }
                
                let strategy = CheatStrategy::new();
                let mut player = Player::new(interpreter, strategy, display);
                player.set_max_turns(max_turns);
                
                let result = player.play_game(program).await?;
                let turns = player.get_turn_count();
                stats.add_game(result.clone(), turns);
                result
            }
            (InterpreterType::TrekBasic, StrategyType::Random) => {
                let interpreter = TrekBasicInterpreter::new(python_path.clone(), trekbasic_path.clone());
                let strategy = RandomStrategy::new();
                let mut player = Player::new(interpreter, strategy, display);
                player.set_max_turns(max_turns);
                
                let result = player.play_game(program).await?;
                let turns = player.get_turn_count();
                stats.add_game(result.clone(), turns);
                result
            }
            (InterpreterType::TrekBasic, StrategyType::Cheat) => {
                let interpreter = TrekBasicInterpreter::new(python_path.clone(), trekbasic_path.clone());
                let strategy = CheatStrategy::new();
                let mut player = Player::new(interpreter, strategy, display);
                player.set_max_turns(max_turns);
                
                let result = player.play_game(program).await?;
                let turns = player.get_turn_count();
                stats.add_game(result.clone(), turns);
                result
            }
            (InterpreterType::TrekBasicJ, StrategyType::Random) => {
                let interpreter = TrekBasicJInterpreter::new(java_path.clone(), trekbasicj_path.clone());
                let strategy = RandomStrategy::new();
                let mut player = Player::new(interpreter, strategy, display);
                player.set_max_turns(max_turns);
                
                let result = player.play_game(program).await?;
                let turns = player.get_turn_count();
                stats.add_game(result.clone(), turns);
                result
            }
            (InterpreterType::TrekBasicJ, StrategyType::Cheat) => {
                let interpreter = TrekBasicJInterpreter::new(java_path.clone(), trekbasicj_path.clone());
                let strategy = CheatStrategy::new();
                let mut player = Player::new(interpreter, strategy, display);
                player.set_max_turns(max_turns);
                
                let result = player.play_game(program).await?;
                let turns = player.get_turn_count();
                stats.add_game(result.clone(), turns);
                result
            }
        };
        
        println!("  Result: {}", result.description());
    }
    
    stats.print_summary();
    Ok(())
} 