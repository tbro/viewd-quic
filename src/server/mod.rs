mod controller;
mod navigator;
mod cursor;
mod window;
mod handlers;
mod quic_service;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use bytes::Bytes;
use anyhow::Result;
use crossbeam_channel::unbounded;

use crate::{ server::controller::Controller, server::quic_service::QuicService};

/// Viewd Server to handle network requests and issue commands to SDL2
pub struct Server {
    quic: QuicService,
    control: Controller,
    exiting: Arc<Mutex<bool>>,
}

impl Server {
    pub fn new(bind: String, path: &Path) -> Result<Self> {
        let (tx_res, rx_res) = unbounded::<Bytes>();
        let (tx_req, rx_req) = unbounded::<Bytes>();
        let exiting = Arc::new(Mutex::new(false));
        let mut control = Controller::new(path, rx_req, tx_res, exiting.clone())?;
        let quic = QuicService::new(bind, tx_req, rx_res)?;
	control.next()?;
        let s = Server {
            quic,
            control,
            exiting,
        };
        Ok(s)
    }
    /// start event loop
    pub fn run(mut self) -> Result<()> {
        // listen for network connections
        self.quic.listen_task();
        loop {
            if *self.exiting.lock().unwrap() {
                break Ok(());
            }
            // send commands to the controller
            self.control.handle_request()?;
            // register window events
            self.control.handle_events();
        }
    }
}

