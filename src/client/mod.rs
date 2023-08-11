mod term_view;
mod quic_service;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, TryRecvError};
use terminal_keycode::KeyCode;
use tracing::debug;

use crate::model::Request;
use crate::client::quic_service::QuicService;
use crate::client::term_view::{TermView, TermInput};

// TODO organize / cleanup the client

/// Runs a client
pub async fn run_client(host: String) -> Result<(), Box<dyn Error>> {
    let client = QuicService::new(host)?;
    // connect to server, get receive and send channels to server
    let (receive, mut send) = client.connect().await?;
    let view = TermView::new()?;
    // spawn a task that copies responses from the server to stdout
    let _handle_out = view.stdout_task(receive);
    // track if we are exiting
    let should_exit = Arc::new(Mutex::new(false));
    let (tx, rx) = mpsc::channel::<KeyCode>();

    let input = TermInput::new()?;
    let handle = input.stdin_task(should_exit.clone(), tx);

    // handle incoming keycodes
    loop {
        if *should_exit.lock().expect("lock mutex") {
            break;
        }
        match rx.try_recv() {
            Ok(keycode) => {
                match keycode {
                    KeyCode::Char('q') |
		    KeyCode::Escape | 
		    KeyCode::CtrlC => *should_exit.lock().expect("lock mutex") = true,
		    // If not a Client command send Request to Server
                    _ => {
			if let Some(request) = Request::new(keycode) {
			    let bytes = request.to_bytes()?;
			    send.send(bytes.into()).await?;
			    
			} else {
			    debug!("keycode does not represent a server command");
			}
                    }
                };
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        }
    }

    handle.await.expect("join mpsc handle")?;
    
    Ok(())
}

