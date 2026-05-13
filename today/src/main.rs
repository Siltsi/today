use dirs;
use std::fs;
use std::path::PathBuf;
use today::events::Category;

use chrono::{Datelike, Local};
use clap::{Parser, Subcommand};

use today::events::{Event, parse_excludes};
use today::filters::{EventFilter, FilterBuilder};
use today::{
    Config, add_event, birthday::handle_birthday, create_providers, events::MonthDay, run,
};

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// List all event providers
    Providers,

    /// Adds an event to an event provider
    Add {
        #[arg(short, long, help = "Name of event provider")]
        provider: String,

        #[arg(short, long, help = "Date of event. Format: YYYY-MM-DD")]
        date: String,

        #[arg(short = 'e', long, help = "Description of event")]
        description: String,

        #[arg(short, long, help = "Category of event. Format: primary[/secondary]")]
        category: String,
    },
}

#[derive(Parser)]
#[command(name = " today")]
struct Args {
    #[command(subcommand)]
    cmd: Option<Command>,

    #[arg(short, long, help = "Event date in MM-DD format")]
    date: Option<String>,

    #[arg(short, long, help = "Categories to exclude, comma-separated (a/b,c/d)")]
    exclude: Option<String>,

    #[arg(short, long, help = "No age calculation or birthday message")]
    no_birthday: bool,

    #[arg(short, long, help = "Category of event. Format: primary[/secondary]")]
    category: Option<String>,

    #[arg(short, long, help = "A string to search for")]
    text: Option<String>,
}

// Gets the configuration directory path for `app_name`.
// If the directory does not exist, tries to create it.
// Returns an optional `PathBuf` containing the directory path,
// or `None` if the directory can't be created.
fn get_config_path(app_name: &str) -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join(app_name);
        if !config_path.exists() {
            if let Err(_) = fs::create_dir(&config_path) {
                eprintln!("Unable to create config directory for {}", app_name);
                return None;
            } else {
                println!("Config directory created for {}", app_name);
            }
        }
        return Some(config_path);
    }
    None
}

fn main() {
    let args = Args::parse();

    const APP_NAME: &str = "today";

    let month_day = if let Some(md) = args.date {
        match MonthDay::from_str_split(&md) {
            Ok(md) => md,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    } else {
        let today = Local::now().date_naive();
        MonthDay::new(today.month(), today.day())
    };

    let mut builder = FilterBuilder::new().month_day(month_day);

    if let Some(ctg) = args.category {
        let category = Category::from_str(&ctg);
        builder = builder.category(category);
    }

    if let Some(txt) = args.text {
        builder = builder.text(txt);
    }

    if let Some(excludes) = args.exclude {
        let categories = parse_excludes(&excludes);
        for category in categories {
            builder = builder.exclude(category);
        }
    }

    let filter: EventFilter = builder.build();

    let config_path = match get_config_path(APP_NAME) {
        Some(config_path) => config_path,
        None => {
            eprintln!("Unable to get config path");
            return;
        }
    };
    let toml_path = config_path.join(format!("{}.toml", APP_NAME));
    let config_str = match fs::read_to_string(toml_path) {
        Ok(config_str) => config_str,
        Err(error) => {
            eprintln!("Couldn't read file: {}", error);
            return;
        }
    };
    let config: Config = match toml::from_str(&config_str) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Invalid configuration: {}", error);
            return;
        }
    };

    if !args.no_birthday {
        handle_birthday();
    }

    match args.cmd {
        Some(Command::Providers) => {
            let providers = create_providers(&config, &config_path);
            for provider in providers {
                println!(
                    "{}{}",
                    provider.name(),
                    if provider.is_add_supported() { "*" } else { "" }
                );
            }
        }
        Some(Command::Add {
            provider,
            date,
            description,
            category,
        }) => {
            let date = match chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                Ok(date) => date,
                Err(error) => {
                    eprintln!("Invalid date format: {}", error);
                    return;
                }
            };
            let category = Category::from_str(&category);
            let event = Event::new_singular(date, description, category);

            if let Err(error) = add_event(&config, &config_path, &provider, &event) {
                eprintln!("Unable to add event: {}", error);
                return;
            }
        }
        _ => {
            if let Err(e) = run(&config, &config_path, &filter) {
                eprintln!("Error running program: {}", e);
                return;
            }
        }
    }
}
