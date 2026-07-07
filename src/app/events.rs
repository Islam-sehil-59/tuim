use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyEvent, MouseEvent};

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
}

pub fn next(timeout: Duration) -> io::Result<Option<AppEvent>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key_event) => Ok(Some(AppEvent::Key(key_event))),
        Event::Mouse(mouse_event) => Ok(Some(AppEvent::Mouse(mouse_event))),
        _ => Ok(None),
    }
}
