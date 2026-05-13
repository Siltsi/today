use crate::{events::Event, filters::EventFilter};
use std::{error::Error, fmt};

pub mod csvfile;
pub mod sqlite;
pub mod textfile;
pub mod web;

pub trait EventProvider {
    fn name(&self) -> String;
    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) -> Result<(), EventProviderError>;
    fn is_add_supported(&self) -> bool {
        false
    }
    fn add_event(&self, event: &Event) -> Result<(), EventProviderError>;
}

#[derive(Debug)]
pub enum EventProviderError {
    Io(String),
    Parse(String),
    Db(String),
    Http(String),
    OperationNotSupported,
    OperationFailed(String),
}

impl fmt::Display for EventProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventProviderError::Io(message) => write!(f, "I/O error: {}", message),
            EventProviderError::Parse(message) => write!(f, "Parse error: {}", message),
            EventProviderError::Db(message) => write!(f, "Database error: {}", message),
            EventProviderError::Http(message) => write!(f, "HTTP error: {}", message),
            EventProviderError::OperationNotSupported => write!(f, "operation not supported"),
            EventProviderError::OperationFailed(message) => write!(f, "operation failed: {}", message),
        }
    }
}

impl Error for EventProviderError {}
