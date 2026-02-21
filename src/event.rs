use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::app::Action;

pub async fn run_event_loop(tx: mpsc::UnboundedSender<Action>) {
    let mut reader = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                if tx.send(Action::Tick).is_err() {
                    break;
                }
            }
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        if let Some(action) = key_to_action(key) {
                            if tx.send(action).is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {
                        // Terminal will re-render on next frame
                    }
                    Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }
}

fn key_to_action(key: KeyEvent) -> Option<Action> {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }

    match key.code {
        KeyCode::Up => Some(Action::Key(KeyAction::Up)),
        KeyCode::Down => Some(Action::Key(KeyAction::Down)),
        KeyCode::Left => Some(Action::Key(KeyAction::Left)),
        KeyCode::Right => Some(Action::Key(KeyAction::Right)),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('d') => Some(Action::Key(KeyAction::Dispatch)),
        KeyCode::Char('m') => Some(Action::Key(KeyAction::ToggleAutoMode)),
        KeyCode::Char('r') => Some(Action::Key(KeyAction::Refresh)),
        KeyCode::Char('c') => Some(Action::Key(KeyAction::ClearAgent)),
        KeyCode::Char('x') => Some(Action::Key(KeyAction::ClearLogs)),
        KeyCode::Char(':') => Some(Action::Key(KeyAction::ActivateInput)),
        KeyCode::Enter => Some(Action::Key(KeyAction::Select)),
        KeyCode::Esc => Some(Action::Key(KeyAction::Escape)),
        KeyCode::Char(c) => Some(Action::Key(KeyAction::Char(c))),
        KeyCode::Backspace => Some(Action::Key(KeyAction::Backspace)),
        KeyCode::Tab => Some(Action::Key(KeyAction::Tab)),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub enum KeyAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Escape,
    Dispatch,
    ToggleAutoMode,
    Refresh,
    ClearAgent,
    ClearLogs,
    ActivateInput,
    Char(char),
    Backspace,
    Tab,
}
