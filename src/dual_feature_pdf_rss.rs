
use std::{collections::HashMap, fs};
use anyhow::{Context, Result};

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

use crate::{explain::explain, file_parse, to_string, BlogBody, PMDPDFSerializer, PMDRSSSerializer, PMDPureTextSerializer, PMDSerializer};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long)] out_dir: Option<String>, 

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Rss  {files: Vec<PathBuf> },

    Pdf   {files: Vec<PathBuf> },
    
    Paragraph {file: PathBuf },
    Subtitle  {file: PathBuf },

    Explain {
        feature: Option<String>
    },
}



pub fn execute() -> Result<()> {
    let cli = Cli::parse();
    let dir = cli.out_dir.unwrap_or("./out".into());
    if !Path::new(dir.as_str()).exists() {
        fs::create_dir_all(dir.clone())?; 
    }

    match &cli.command {
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
        #[cfg(feature = "pdf")]
        Commands::Pdf{files} => {
            let out_dir = Path::new(dir.as_str());
            for file in files {
                let stem = file.as_path().file_stem().context("expected file name")?;
                let stem = format!("pdf-{}", stem.to_str().expect("could not convert filename to str"));
                let mut out_file = out_dir.join(stem.clone());
                out_file.set_extension("pdf");
                println!("outputting to file {}", out_file.to_str().expect("whatever"));

                let result = file_parse(&file.to_str().context("expected a file")?.to_string())?;
                let html   = to_string(&result, PMDPDFSerializer::new(stem.as_str()))?;
                if out_file.exists() {
                    fs::remove_file(&out_file)?;
                }
                fs::write(out_file, html)?;
            }
        }, 
        Commands::Rss{files}  => {
            let out_dir = Path::new(dir.as_str());
            for file in files {
                let stem = file.as_path().file_stem().context("expected file name")?;
                let mut out_file = out_dir.join(stem);
                out_file.set_extension("rss");

                let result = file_parse(&file.to_str().context("expected a file")?.to_string())?;
                let html   = to_string(&result, PMDRSSSerializer::new(stem.to_str().context("converting OsStr to str")?))?;
                if out_file.exists() {
                    fs::remove_file(&out_file)?;
                }
                fs::write(out_file, html)?;
            }
        }, 
        Commands::Explain{feature}  => {
            explain(feature);
        },
    }
    Ok(())
}
