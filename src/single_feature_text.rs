
use std::{collections::HashMap, fs};
use anyhow::{Context, Result};

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use crate::{explain::explain, file_parse, to_string, BlogBody, PMDPureTextSerializer, PMDSerializer};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)] out_dir: Option<String>, 

    #[command(subcommand)]
    command: Option<Commands>,

    files: Vec<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Explain {
        feature: Option<String>
    },
    
    Paragraph {file: PathBuf },
    Subtitle  {file: PathBuf },
}

pub fn execute() -> Result<()> {
    let cli = Cli::parse();
    let dir = cli.out_dir.unwrap_or("./out".into());
    if !Path::new(dir.as_str()).exists() {
        fs::create_dir_all(dir.clone())?; 
    }

    if let Some(command) = &cli.command {
        match command {
            Commands::Paragraph { file } => {
                let result = file_parse(&file.to_str().context("expected a file")?.to_string())?;
                let paragraph = result.body.iter().find(|&x| match x { BlogBody::Paragraph(_) => true, _ => false});
                if let Some(BlogBody::Paragraph(content)) = paragraph {
                    let mut serialiser = PMDPureTextSerializer::new();
                    serialiser.references = result.references.clone();
                    let text = serialiser.convert_paragraph(content)?;
                    println!("{text}");
                } else {
                }
            },
            Commands::Subtitle { file } => {
                let result = file_parse(&file.to_str().context("expected a file")?.to_string())?;
                let text = result.header.subtitle;
                println!("{text}");
            },
            Commands::Explain{feature}  => {
                explain(feature);
            }
        }
    } else {
        for file in cli.files {
            let file_path_string = file.as_path().to_str().expect("expected a valid path");
            println!("// {} //////////////////////////////////////////////////////////////////////////", file_path_string);

            let result = file_parse(&file.to_str().context("expected a file")?.to_string())?;
            let text   = to_string(&result, PMDPureTextSerializer::new())?;

            println!("{text}");

            println!("////////////////////////////////////////////////////////////////////////////////");
        }
    }

    Ok(())
}
