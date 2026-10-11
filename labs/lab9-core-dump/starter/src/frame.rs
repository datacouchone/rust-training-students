//! Wire framing and the metric command parser.
//!
//! Frame format: a 4-byte big-endian length prefix followed by that many
//! bytes of UTF-8 text. The text body of a request frame is expected to be
//! `METRIC <name> <value>`; the server answers with a response frame whose
//! body is `OK` or `ERR <reason>`.

use std::fmt;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Largest frame body we will accept, in bytes. Guards against a garbled or
/// hostile length prefix causing an unbounded allocation.
pub const MAX_FRAME_LEN: u32 = 64 * 1024;

/// A single parsed `METRIC <name> <value>` command.
#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
}

/// Why a frame failed to parse as a metric command. Every variant is a
/// well-formed `Result::Err` -- [`parse_metric_frame`] is a total function
/// that never panics, no matter what bytes it is given.
#[derive(Debug, Clone, PartialEq)]
pub enum FrameError {
    /// The frame was empty or whitespace-only.
    Empty,
    /// The first token wasn't `METRIC`.
    UnknownCommand(String),
    /// No name token after `METRIC`.
    MissingName,
    /// No value token after the name.
    MissingValue,
    /// Extra tokens after the value.
    TrailingTokens,
    /// The value token wasn't a valid `f64`.
    InvalidValue(String),
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Empty => write!(f, "empty frame"),
            FrameError::UnknownCommand(cmd) => {
                write!(f, "unknown command {cmd:?}, expected METRIC")
            }
            FrameError::MissingName => write!(f, "missing metric name"),
            FrameError::MissingValue => write!(f, "missing metric value"),
            FrameError::TrailingTokens => write!(f, "unexpected extra tokens after value"),
            FrameError::InvalidValue(v) => write!(f, "value {v:?} is not a valid f64"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Parse a `METRIC <name> <value>` text frame.
///
/// This is a total function: it never panics and always returns `Ok` or a
/// descriptive `Err`. Whether a malformed frame is *handled* gracefully or
/// allowed to panic the caller is a decision made by the connection
/// handler (see `server::process_frame` and the `panic` fault), not here.
pub fn parse_metric_frame(bytes: &[u8]) -> Result<Metric, FrameError> {
    let text = String::from_utf8_lossy(bytes);
    let mut tokens = text.split_whitespace();

    let command = tokens.next().ok_or(FrameError::Empty)?;
    if command != "METRIC" {
        return Err(FrameError::UnknownCommand(command.to_string()));
    }

    let name = tokens.next().ok_or(FrameError::MissingName)?;
    let raw_value = tokens.next().ok_or(FrameError::MissingValue)?;

    if tokens.next().is_some() {
        return Err(FrameError::TrailingTokens);
    }

    let value: f64 = raw_value
        .parse()
        .map_err(|_| FrameError::InvalidValue(raw_value.to_string()))?;

    Ok(Metric {
        name: name.to_string(),
        value,
    })
}

/// Read one length-prefixed frame. Returns `Ok(None)` on a clean EOF that
/// arrives before any bytes of a new frame (the peer closed the
/// connection between frames, not mid-frame).
pub async fn read_frame<R>(reader: &mut R) -> io::Result<Option<Vec<u8>>>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let len = u32::from_be_bytes(len_buf);
    if len > MAX_FRAME_LEN {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("frame length {len} exceeds max {MAX_FRAME_LEN}"),
        ));
    }

    let mut body = vec![0u8; len as usize];
    reader.read_exact(&mut body).await?;
    Ok(Some(body))
}

/// Write one length-prefixed frame and flush it.
pub async fn write_frame<W>(writer: &mut W, body: &[u8]) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let len = u32::try_from(body.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "frame body too large to encode",
        )
    })?;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(body).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_parser_rejects_malformed() {
        let cases: &[&[u8]] = &[
            b"",
            b"   ",
            b"METRIC",
            b"METRIC cpu.load",
            b"METRIC cpu.load not_a_number",
            b"METRIC cpu.load 0.42 extra_token",
            b"GAUGE cpu.load 0.42",
            b"metric cpu.load 0.42",
        ];
        for bytes in cases {
            let result = parse_metric_frame(bytes);
            assert!(
                result.is_err(),
                "expected an error for input {:?}, got {:?}",
                String::from_utf8_lossy(bytes),
                result
            );
        }
    }

    #[test]
    fn parses_well_formed_frame() {
        let metric = parse_metric_frame(b"METRIC cpu.load 0.42").expect("should parse");
        assert_eq!(metric.name, "cpu.load");
        assert_eq!(metric.value, 0.42);
    }
}
