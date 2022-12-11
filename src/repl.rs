use core::result::Result;
use expectrl::{Captures, Error, Session};
use regex::Regex;

use futures_lite::{AsyncRead, AsyncWrite};

fn trim_last_newline<'a>(buf: &'a [u8]) -> &'a [u8] {
    if buf.len() > 0 && buf[buf.len() - 1] == b'\n' {
        if buf.len() > 1 && buf[buf.len() - 2] == b'\r' {
            &buf[..buf.len() - 2]
        } else {
            &buf[..buf.len() - 1]
        }
    } else {
        &buf
    }
}

pub struct ReplWrapper<P, S> {
    pub session: Session<P, S>,
    pub prompt: Regex,
}

impl<P, S> ReplWrapper<P, S> {
    pub fn new(session: Session<P, S>, prompt: Regex) -> Self {
        ReplWrapper { session, prompt }
    }
}

impl<P, S: AsyncRead + AsyncWrite + Unpin> ReplWrapper<P, S> {
    pub async fn expect_prompt(&mut self) -> Result<(), Error> {
        self._expect_prompt().await?;
        Ok(())
    }

    async fn _expect_prompt(&mut self) -> Result<Captures, Error> {
        let prompt = self.prompt.as_str().to_string();
        self.expect(expectrl::Regex(prompt)).await
    }

    /// Run command and return raw output
    pub async fn run_command_raw<SS: AsRef<str> + Clone>(
        &mut self,
        cmd: SS,
    ) -> Result<Captures, Error> {
        self.send_line(cmd).await?;
        self._expect_prompt().await
    }

    /// Run command and return trimmed output
    pub async fn run_command<SS: AsRef<str> + Clone>(&mut self, cmd: SS) -> Result<Vec<u8>, Error> {
        let m = self.run_command_raw(cmd).await?;
        let recv = m.before();

        // trim last \r\n or \n
        Ok(trim_last_newline(recv).to_vec())
    }

    pub async fn send_line(&mut self, line: impl AsRef<str>) -> Result<(), Error> {
        self.session.send_line(line.as_ref()).await?;
        self.expect(line.as_ref()).await?;

        // expectrl::Session append \r\n
        match self.expect("\r\n").await {
            Ok(_) => Ok(()),
            Err(expectrl::Error::ExpectTimeout) => {
                // Sometimes not echo back
                if self.is_matched("\n").await? {
                    self.expect("\n").await?;
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl<P, S> std::convert::Into<Session<P, S>> for ReplWrapper<P, S> {
    fn into(self) -> Session<P, S> {
        self.session
    }
}

impl<P, S> std::ops::Deref for ReplWrapper<P, S> {
    type Target = Session<P, S>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl<P, S> std::ops::DerefMut for ReplWrapper<P, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_last_newline() {
        assert_eq!(trim_last_newline(b"123"), b"123");
        assert_eq!(trim_last_newline(b"123\n"), b"123");
        assert_eq!(trim_last_newline(b"123\r\n"), b"123");

        assert_eq!(trim_last_newline(b"12\n3"), b"12\n3");
        assert_eq!(trim_last_newline(b"12\n3\n"), b"12\n3");
        assert_eq!(trim_last_newline(b"12\n3\r\n"), b"12\n3");

        assert_eq!(trim_last_newline(b""), b"");
        assert_eq!(trim_last_newline(b" "), b" ");
        assert_eq!(trim_last_newline(b"\n"), b"");
        assert_eq!(trim_last_newline(b"\r\n"), b"");

        assert_eq!(trim_last_newline(b"\n\r\n"), b"\n");
    }
}
