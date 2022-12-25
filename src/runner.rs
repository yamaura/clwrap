use crate::{Error, Session};
use futures_lite::{AsyncRead, AsyncWrite};
use std::time::Duration;
use tracing::{trace, warn};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct UserPassLoginRunner {
    prompt_username: String,
    prompt_password: String,

    #[builder(default = Some(3))]
    retry_prompt_username: Option<u64>,
}

impl UserPassLoginRunner {
    pub async fn login<P, S: AsyncRead + AsyncWrite + Unpin>(
        self,
        mut session: Session<P, S>,
        username: &str,
        password: &str,
        prompt: Option<&regex::Regex>,
    ) -> core::result::Result<Session<P, S>, Error> {
        // First time is quick
        session.set_expect_timeout(Some(Duration::from_millis(1)));

        trace!("Start expect prompt username");
        let mut counter = 0;
        while counter < self.retry_prompt_username.unwrap_or(1) {
            trace!("counter={}: try..", counter);
            match session.expect(self.prompt_username.clone()).await {
                Ok(_) => break,
                Err(expectrl::Error::ExpectTimeout) => {
                    if let Some(prompt) = prompt {
                        // already login check
                        let prompt = prompt.as_str().to_string();
                        if session.is_matched(expectrl::Regex(prompt)).await? {
                            trace!("Already logged in");
                            return Ok(session);
                        }
                    }
                    trace!("send new line");
                    session.send_line("").await?;
                }
                Err(e) => return Err(e.into()),
            }
            session.set_expect_timeout(Some(Duration::from_secs(1)));

            if self.retry_prompt_username.is_some() {
                counter += 1;
            }
        }

        // Set to normal operation
        session.set_expect_timeout(Some(Duration::from_secs(1)));

        trace!("send username");
        session.send_line(username).await?;
        if let Err(e) = session.expect(self.prompt_password).await {
            warn!("Could not found password prompt");
            return Err(e.into());
        };
        trace!("send password");
        session.send_line(password).await?;

        Ok(session)
    }
}

#[derive(TypedBuilder)]
#[builder(build_method(vis = "", name = build_inner))]
pub struct LinuxLoginRunner {
    #[builder(default = Some(Duration::from_secs_f32(3.)))]
    timeout: Option<Duration>,
}

impl LinuxLoginRunnerBuilder<((),)> {
    pub fn build(self) -> UserPassLoginRunner {
        let runner = self.build_inner();
        UserPassLoginRunner::builder()
            .prompt_username("login: ".to_string())
            .prompt_password("Password:".to_string())
            .retry_prompt_username(runner.timeout.map(|t| t.as_secs() + 1))
            .build()
    }
}
