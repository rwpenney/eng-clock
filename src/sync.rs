/*
 *  Clock offset estimation tools for eng-clock
 *  RW Penney, May 2023
 */

use std::sync::mpsc;
use std::thread;
use crate::{ OffsetEvent, UImessage, UIsender };


pub struct OffsetEstimator {
    tkr_channel: mpsc::Sender<OffsetEvent>,
    ui_channel: UIsender
}

impl OffsetEstimator {
    pub fn new(tkr_channel: mpsc::Sender<OffsetEvent>,
               ui_channel: UIsender) -> OffsetEstimator {
        OffsetEstimator {
            tkr_channel,
            ui_channel
        }
    }

    /// Entry-point for clock-offset thread communicating via message queues
    pub fn run(&self) {
        loop {
            thread::sleep(std::time::Duration::from_millis(1500));
            let offs = OffsetEvent {};
            self.tkr_channel.send(offs).unwrap();
            self.ui_channel.send(UImessage::Offset(offs)).unwrap();
        }
    }
}

// (C)Copyright 2023, RW Penney
