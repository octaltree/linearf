use log::LevelFilter;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "linearfinder")]
struct Opt {
    /// file logging
    #[structopt(long, name = "filename")]
    log: Option<String>
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    if let Some(name) = &opt.log {
        simple_logging::log_to_file(name, LevelFilter::Trace)?;
    }
    println!("Hello, world!");
    Ok(())
}
