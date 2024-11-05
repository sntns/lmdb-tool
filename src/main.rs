use clap::{builder::OsStr, Parser};

mod lmdb;

#[derive(Parser, Debug, Clone)]
#[clap(name = "lmbd-tool", version, author, about)]
pub struct Cli {
    #[clap(short, long, value_name = "FILE")]
    input: std::path::PathBuf,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long, default_value = "error,lmbd::database=debug")]
    trace: String,

    #[clap(subcommand)]
    command: Commands,
}

impl From<lmdb::WordSize> for OsStr {
    fn from(size: lmdb::WordSize) -> Self {
        match size {
            lmdb::WordSize::Word32 => "32".into(),
            lmdb::WordSize::Word64 => "64".into(),
        }
    }
}

#[derive(Parser, Debug, Clone)]
enum Commands {
    Convert {
        #[clap(short, long, default_value = lmdb::WordSize::Word64)]
        format: lmdb::WordSize,

        #[clap(short, long, value_name = "FILE")]
        output: std::path::PathBuf,
    },
    Dump,
    Info,
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
        Commands::Convert { format, output } => {
            println!("Converting to {:?}", format);
            let mut db_in = lmdb::Factory::open(opts.input.clone()).unwrap();
            let mut cur_in = db_in.read_cursor().unwrap();

            let mut db_out = lmdb::Factory::create(output.clone(), format).unwrap();
            let mut cur_out = db_out.write_cursor().unwrap();

            while let Some(element) = cur_in.next().unwrap() {
                cur_out.push_element(element).unwrap();
            }
            cur_out.commit().unwrap();
        }
        Commands::Dump => {
            let mut db = lmdb::Factory::open(opts.input.clone()).unwrap();
            let mut cur = db.read_cursor().unwrap();
            let mut i = 0;
            while let Some(element) = cur.next().unwrap() {
                println!("#{}: {:#?}", i, element);
                i += 1;
            }
        },
        Commands::Info => {
            let wordize = lmdb::Factory::detect(opts.input.clone()).unwrap();
            println!("Word size: {:?}", wordize);
            let db = lmdb::Factory::open(opts.input.clone()).unwrap();
            println!("{:#?}", db.meta);
        }
    }
}
