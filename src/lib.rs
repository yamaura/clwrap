use futures_lite::{AsyncRead, AsyncWrite};
use tracing::trace;

pub mod repl;
pub mod runner;

pub async fn linux_oneshot<
    P,
    S: AsyncRead + AsyncWrite + Unpin,
    SS: AsRef<str> + Clone + std::fmt::Debug,
>(
    session: expectrl::Session<P, S>,
    username: &str,
    password: &str,
    prompt: Option<String>,
    finalize: fn(expectrl::Session<P, S>) -> (),
    cmd: SS,
) -> core::result::Result<Vec<u8>, expectrl::Error> {
    let quit = "exit";
    let prompt = match prompt {
        Some(prompt) => prompt,
        None => r".*]# *".to_string(),
    };
    let prompt = regex::Regex::new(&prompt).unwrap();

    let session = runner::LinuxLoginRunner::builder()
        .build()
        .login(session, username, password, Some(&prompt))
        .await?;
    trace!("successfully logged in");
    let mut session = crate::repl::ReplWrapper::new(session, prompt);
    trace!("expect prompt...");
    session.expect_prompt().await?;
    trace!("run_command: {:?}", cmd);
    let recv = session.run_command(cmd).await;

    trace!("send quit command: {}", quit);
    session.send_line(quit).await?;

    trace!("finalize...");
    finalize(session.into());
    recv
}
