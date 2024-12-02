use base64::engine::general_purpose::STANDARD as base64;
use base64::Engine;
use json;
use std::{collections::HashMap, vec};

use clap::{builder::OsStr, Parser};

mod lmdb;

#[derive(Parser, Debug, Clone)]
#[clap(name = "lmbd", version, author, about)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long, default_value = "error,lmbd::database=debug")]
    trace: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug, Clone)]
enum Commands {
    #[clap(
        about = "Convert a database to another bitsize format. This is useful for converting a 32-bit database to a 64-bit database without losing data."
    )]
    Convert {
        #[clap(value_name = "source", help = "The source file to convert")]
        input: std::path::PathBuf,

        #[clap(value_name = "destination", help = "The destination file to write to")]
        output: Option<std::path::PathBuf>,

        #[clap(
            short,
            long,
            default_value = "word64",
            help = "The word size to convert to"
        )]
        format: lmdb::WordSize,
    },
    Dump {
        #[clap(value_name = "file")]
        input: std::path::PathBuf,

        #[clap(long, help = "Convert keys to strings")]
        string_key: bool,

        #[clap(long, help = "Convert values to strings")]
        string_value: bool,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
    Info {
        #[clap(value_name = "file")]
        input: std::path::PathBuf,

        #[arg(long, help = "Output as JSON")]
        json: bool,
    },
}

fn main() {
    let opts = Cli::parse();
    // Setup tracing & logging
    tracing_subscriber::fmt()
        .with_env_filter(opts.clone().trace)
        .with_max_level(match opts.verbose {
            0 => tracing::Level::INFO,
            1 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        })
        .init();

    tracing::debug!("{:#?}", opts.clone());

    match opts.command {
        Commands::Convert {
            input,
            output,
            format,
        } => {
            let output = match output {
                Some(output) => {
                    if input == output {
                        tracing::warn!("Output file is the same as input file, this will cause data loss");
                        tracing::warn!("Please specify a different output file");
                        std::process::exit(1);
                    } else {
                        output
                    }
                },
                None => input.clone(),
            };

            let wordize = lmdb::Factory::detect(input.clone()).unwrap();
            if wordize != format {
                tracing::info!("Converting database from {:?} to {:?}", wordize, format);
                let mut db_in = lmdb::Factory::open(input.clone()).unwrap();
                let mut cur_in = db_in.read_cursor().unwrap();

                let mut db_out = lmdb::Factory::create(output.clone(), format).unwrap();
                let mut cur_out = db_out.write_cursor().unwrap();

                while let Some(mut element) = cur_in.next().unwrap() {
                    if element.value == "null".as_bytes() {
                        element.value = vec![];
                    }
                    cur_out.push_element(element).unwrap();
                }
                cur_out.commit().unwrap();

                db_in.close().unwrap();
                db_out.close().unwrap();

                if input == output {
                    std::fs::rename(output.clone(), input.clone()).unwrap();
                }
            } else if input != output {
                tracing::info!("No conversion needed, copying file");
                std::fs::copy(input.clone(), output.clone()).unwrap();
            } else {
                tracing::info!("No conversion needed");
            }
        }
        Commands::Dump {
            input,
            string_key,
            string_value,
            json,
        } => {
            let mut db = lmdb::Factory::open(input.clone()).unwrap();
            let mut cur = db.read_cursor().unwrap();
            let items: HashMap<String, String> = cur
                .next()
                .unwrap()
                .iter()
                .map(|element| {
                    let key = if string_key {
                        String::from_utf8_lossy(&element.key).to_string()
                    } else {
                        base64.encode(&element.key)
                    };
                    let value = if string_value {
                        String::from_utf8_lossy(&element.value).to_string()
                    } else {
                        base64.encode(&element.value)
                    };
                    (key, value)
                })
                .collect();

            if json {
                let json_string = json::stringify_pretty(items, 2);
                println!("{}", json_string);
                return;
            }

            for (key, value) in items {
                println!("{}: {}", key, value);
            }
        }
        Commands::Info { input, json } => {
            let wordize = lmdb::Factory::detect(input.clone()).unwrap();
            let db = lmdb::Factory::open(input.clone()).unwrap();
            let out = json::object! {
                    "word-size": Into::<u8>::into(wordize),
                    "pages": json::object! {
                        "leaf": db.meta.main.leaf_pages,
                        "branch": db.meta.main.branch_pages,
                        "overflow": db.meta.main.overflow_pages,
                    },
                    "root": db.meta.main.root,
                    "last": db.meta.last_pgno,
                    "entries": db.meta.main.entries,
            };
            if json {
                println!("{}", json::stringify_pretty(out, 2));
                return;
            }
            println!("Word size: {:?}", wordize);
            println!(
                "Pages: leaf:{:?}, branch:{:?}, overflow:{:?}",
                db.meta.main.leaf_pages, db.meta.main.branch_pages, db.meta.main.overflow_pages
            );
            println!("Root: {:?}", db.meta.main.root);
            println!("Last: {:?}", db.meta.last_pgno);
            println!("Entries: {:?}", db.meta.main.entries);
        }
    }
}
