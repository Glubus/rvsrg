//! Shared channel infrastructure between system threads.

use crate::input::events::{GameAction, InputCommand, RawInputEvent};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
// Import snapshots for render handoff.
use crate::shared::snapshot::RenderState;

#[derive(Debug, Clone)]
pub enum SystemEvent {
    Resize { width: u32, height: u32 },
    FocusLost,
    FocusGained,
    Quit,
}

/// Aggregates the cross-thread channels.
#[derive(Clone)]
pub struct SystemBus {
    // Main -> Input (raw key events)
    pub raw_input_tx: Sender<RawInputEvent>,
    pub raw_input_rx: Receiver<RawInputEvent>,

    // Commands sent to the input thread
    pub input_cmd_tx: Sender<InputCommand>,
    pub input_cmd_rx: Receiver<InputCommand>,

    // Input -> Logic (gameplay actions)
    pub action_tx: Sender<GameAction>,
    pub action_rx: Receiver<GameAction>,

    // Logic -> Render (snapshots)
    pub render_tx: Sender<RenderState>,
    pub render_rx: Receiver<RenderState>,

    // Main -> Logic (system events)
    pub sys_tx: Sender<SystemEvent>,
    pub sys_rx: Receiver<SystemEvent>,
}

impl SystemBus {
    pub fn new() -> Self {
        let (raw_input_tx, raw_input_rx) = unbounded();
        let (input_cmd_tx, input_cmd_rx) = unbounded();
        let (action_tx, action_rx) = unbounded();

        // Bounded render channel: max 2 frames queued to limit latency.
        let (render_tx, render_rx) = bounded(2);

        let (sys_tx, sys_rx) = unbounded();

        Self {
            raw_input_tx,
            raw_input_rx,
            input_cmd_tx,
            input_cmd_rx,
            action_tx,
            action_rx,
            render_tx,
            render_rx, // Bind bounded channel endpoints
            sys_tx,
            sys_rx,
        }
    }
}
