use anyhow::Result;
use clap::Parser;
use expectrl;
use std::env;
use tracing::debug;

/// ConsoLe wrapper runner
///
/// Auto login then execute command
/// Currently support only for linux system
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    #[arg(short = 'u', long)]
    username: String,

    #[arg(short = 'p', long)]
    password: String,
}

fn main() -> Result<()> {
    {
        use tracing_subscriber::{/*filter::LevelFilter,*/ EnvFilter};
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| {
                        EnvFilter::builder()
                            .parse("clwrap=warn")
                            .expect("Envfilter")
                    })
                    // Use default
                    // .add_directive(LevelFilter::WARN.into()),
            )
            .init();
    }
    futures_lite::future::block_on(async {
        let mut argv = env::args_os();
        let args = Args::parse_from(argv.by_ref().take_while(|arg| arg != "--"));
        let spawn_command = argv
            .by_ref()
            .take_while(|arg| arg != "--")
            .collect::<Vec<_>>();
        let exec_command = argv.collect::<Vec<_>>();

        let spawn_command = spawn_command
            .iter()
            .map(|s| s.to_str().unwrap_or(""))
            .collect::<Vec<&str>>()
            .join(" ");
        let exec_command = exec_command
            .iter()
            .map(|s| s.to_str().unwrap_or(""))
            .collect::<Vec<&str>>()
            .join(" ");

        debug!("{:?}", args);
        debug!("spawn_command: {:?}", spawn_command);
        debug!("exec_command: {:?}", exec_command);

        let username = args.username;
        let password = args.password;

        let session = expectrl::spawn(spawn_command)?;
        let recv = clwrap::linux_oneshot(session, &username, &password, None, |_| (), exec_command)
            .await?;
        let recv = std::str::from_utf8(&recv)?;
        let recv = recv.replace("\r\n", "\n");

        println!("{}", recv);
        Ok(())
    })
}
