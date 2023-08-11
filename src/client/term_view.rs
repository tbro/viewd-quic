use console::Term;
use anyhow::Result;
use s2n_quic::stream::ReceiveStream;
use tokio::io::{AsyncWriteExt, Stdout};
use std::os::unix::prelude::OsStrExt;
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::io::{Stdin, Read};
use raw_tty::{IntoRawMode, RawReader};
use terminal_keycode::{Decoder, KeyCode};
use tokio::task::JoinHandle;

use crate::image::Image;
use crate::model::Response;

/// Handles terminal output
pub struct TermView {
    term: Term,
    stdout: Stdout,
}
impl TermView {
    pub fn new() -> Result<TermView> {
	let stdout = tokio::io::stdout();
	let term = Term::stdout();
	term.clear_screen()?;
	term.write_line("Viewd!")?;
	term.write_line("\r--------")?;
	let view = TermView { term, stdout };
	Ok(view)
    }
    /// Accepts some text bytes and handles writing to stdout
    /// with some formatting.
    pub async fn write_line(&mut self, line: &[u8]) -> Result<()> {
	self.term.clear_line()?;
	let margin = b"\t";
	self.stdout.write_all(&[margin, line].concat()).await?;
	self.stdout.flush().await?;
	Ok(())
    }
    /// Spawn a task to handle writes
    pub fn stdout_task(mut self, mut stream: ReceiveStream) -> JoinHandle<Result<()>> {
	tokio::spawn(async move {
	    while let Ok(Some(data)) = stream.receive().await {
		let response = Response::from_bytes(data)?;
		let path = if let Some(path) = response.path() {
		   path 
		} else {
		    unimplemented!()
		};
		if let Some(image) = Image::new(path) {
		    self.write_line(image.name().as_bytes()).await?;
		};
	    }
	    Ok(())
	})
    }
}

/// Encapsulates terminal input
pub struct TermInput {
    stdin: RawReader<Stdin>,
    decoder: Decoder,
}
impl TermInput {
    pub fn new () -> Result<TermInput > {
	let stdin = std::io::stdin().into_raw_mode()?;
	let decoder = Decoder::new();
	let ti = TermInput { stdin, decoder };
	Ok(ti)
    }
    /// Spawns a task to accept input. Accepts `should_accept` to
    /// gracefully shutdown the loop. Sends keycodes down the mpsc channel.
    pub fn stdin_task(
	mut self,
	should_exit: Arc<Mutex<bool>>,
	tx: Sender<KeyCode>,
    ) -> JoinHandle<Result<()>> {
	tokio::spawn(async move {
            loop {
		if *should_exit.lock().unwrap() {
                    break;
		}
		let mut buf = vec![0];
		self.stdin.read_exact(&mut buf)?;
		for keycode in self.decoder.write(buf[0]) {
                    tx.send(keycode)?;
		}
            }
            Ok(())
	})
    }
}
