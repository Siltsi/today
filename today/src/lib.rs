pub mod birthday;
pub mod events;
pub mod filters;
mod providers;

use std::error::Error;
use std::path::Path;

use serde::Deserialize;

use crate::events::Event;
use crate::filters::EventFilter;
use crate::providers::{
    csvfile::CSVFileProvider,
    sqlite::SQLiteProvider,
    textfile::TextFileProvider,
    web::WebProvider,
    EventProviderError,
};
use providers::EventProvider;

#[derive(Deserialize, Debug)]
pub struct ProviderConfig {
    name: String,
    kind: String,
    resource: String,
}

impl ProviderConfig {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
    providers: Vec<ProviderConfig>,
}

impl Config {
    pub fn providers(&self) -> &Vec<ProviderConfig> {
        &self.providers
    }
}

pub fn create_providers(config: &Config, config_path: &Path) -> Vec<Box<dyn EventProvider>> {
    // Try to create all the event providers specified in `config`.
    // Put them in a vector of trait objects.
    let mut providers: Vec<Box<dyn EventProvider>> = Vec::new();
    for cfg in config.providers.iter() {
        let path = config_path.join(&cfg.resource);
        match cfg.kind.as_str() {
            "text" => {
                let provider = TextFileProvider::new(&cfg.name, &path);
                providers.push(Box::new(provider));
            }
            "csv" => {
                let provider = CSVFileProvider::new(&cfg.name, &path);
                providers.push(Box::new(provider));
            }
            "sqlite" => {
                let provider = SQLiteProvider::new(&cfg.name, &path);
                providers.push(Box::new(provider));
            }
            "web" => {
                let provider = WebProvider::new(&cfg.name, &cfg.resource);
                providers.push(Box::new(provider));
            }
            _ => {
                eprintln!("Unable to make provider: {:?}", cfg);
            }
        }
    }

    providers
}

pub fn run(
    config: &Config,
    config_path: &Path,
    filter: &EventFilter,
) -> Result<(), Box<dyn Error>> {
    let mut events: Vec<Event> = Vec::new();

    let providers = create_providers(config, config_path);
    let mut count = 0;

    for provider in providers {
        provider.get_events(&filter, &mut events)?; // polymorphism at work!
        let new_count = events.len();
        println!(
            "Got {} events from provider '{}'",
            new_count - count,
            provider.name()
        );
        count = new_count;
    }

    for event in events {
        println!("{}", event);
    }

    Ok(())
}

pub fn add_event(config: &Config, config_path: &Path, provider_name: &str, event: &Event) -> Result<(), Box<dyn Error>> {
    let providers = create_providers(config, config_path);
    
    // Find provider by name
    let mut provider: Option<&dyn EventProvider> = None;
    for p in &providers {
        if p.name() == provider_name {
            provider = Some(p.as_ref());
            break;
        }
    }

    match provider {
        Some(p) => {
            if p.is_add_supported() {
                p.add_event(event)?;
                Ok(())
            } else {
                Err(Box::new(EventProviderError::OperationNotSupported))
            }
        }
        None => Err(Box::new(EventProviderError::OperationFailed(
            format!("Unknown event provider '{}'", provider_name),
        ))),
    }
}
