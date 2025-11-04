use anyhow::Result;
use clap::{Parser, Subcommand};

mod cli;
mod compiler;
mod database;

#[derive(Parser)]
#[command(name = "capsule")]
#[command(about = "CapsuleOS Build Toolchain", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiler operations
    Compile {
        /// Source file to compile
        #[arg(short, long)]
        input: String,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Database operations
    Database {
        #[command(subcommand)]
        db_command: DatabaseCommands,
    },
    
    /// System information
    Info,
}

#[derive(Subcommand)]
enum DatabaseCommands {
    /// Initialize a new graph database
    Init {
        /// Database path
        path: String,
    },
    
    /// Query the graph database
    Query {
        /// Query string
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, output } => {
            cli::handle_compile(&input, output.as_deref()).await?;
        }
        Commands::Database { db_command } => {
            match db_command {
                DatabaseCommands::Init { path } => {
                    cli::handle_db_init(&path).await?;
                }
                DatabaseCommands::Query { query } => {
                    cli::handle_db_query(&query).await?;
                }
            }
        }
        Commands::Info => {
            cli::handle_info().await?;
        }
    }
    
    Ok(())
}
