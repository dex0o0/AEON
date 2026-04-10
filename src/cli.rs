use clap::{Parser,Subcommand,CommandFactory,Args};
use std::{fs,env,path::PathBuf};
use serde::{Serialize,Deserialize};
use serde_json;
#[derive(Parser)]
#[command(name = "aeoncli")]
#[command(version = "0.1.0")]
struct Cli{
    #[command(subcommand)]
    command:Command,
}

#[derive(Subcommand)]
enum Command{
        
}

#[tokio::main]
async fn main(){

}


