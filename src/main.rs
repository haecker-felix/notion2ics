use chrono::DateTime;
use icalendar::Calendar;
use serde_path_to_error::Error;

use surf::Url;
use surf::{Client, Config};

use clap::{command, Parser};
use humantime::Duration;

mod notion;
use notion::*;

mod date;
use date::*;

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, required = true, help("Notion integration bearer token"))]
    api_token: String,

    #[arg(long, required = true, help("Notion database uuid"))]
    database: Vec<String>,

    #[arg(
        long,
        default_value = "./",
        help("Path to place the resulting *.ics files")
    )]
    output_path: String,

    #[arg(long, default_value = "15 min", help("Refresh intervall"))]
    refresh_intervall: Duration,
}

#[async_std::main]
async fn main() {
    // Notion API endpoint
    let notion_api = Url::parse("https://api.notion.com/v1/").unwrap();
    let notion_api_version = "2022-06-28";

    // Parse CLI arguments
    let args = Cli::parse();

    // Setup HTTP client
    let config = Config::new()
        .add_header("Authorization", format!("Bearer {}", args.api_token))
        .unwrap()
        .add_header("Notion-Version", notion_api_version)
        .unwrap()
        .add_header("Content-Type", "application/json")
        .unwrap()
        .set_base_url(notion_api);
    let client: Client = config.try_into().unwrap();

    loop {
        // Iterate through database ids
        for database_id in &args.database {
            let entries = notion_query_database(&client, database_id).await;
            create_ics(
                &entries,
                &format!("{}{}.ics", args.output_path, database_id),
            );
        }

        println!(
            "Going to wait {} till next refresh.",
            args.refresh_intervall
        );
        std::thread::sleep(args.refresh_intervall.into());
    }
}

async fn notion_query_database(client: &Client, database_id: &str) -> Vec<DateEntry> {
    println!("Query database: {database_id}");

    let db_uri = format!("databases/{database_id}/query");
    let mut res = client.post(db_uri).body_string("{}".into()).await.unwrap();
    let content = res.body_string().await.unwrap();

    let deserializer = &mut serde_json::Deserializer::from_str(&content);
    let response: NotionDatabaseResponse = serde_path_to_error::deserialize(deserializer).unwrap();

    let mut date_entries: Vec<DateEntry> = Vec::new();
    for database_page in &response.pages {
        let mut date: Option<Date> = None;
        let mut title_prefix = String::new();

        // Optional additional information which gets set as description in ics event
        let mut additional = BTreeMap::new();

        for (prop_name, prop) in &database_page.properties {
            let prop_name = prop_name.clone();
            let mut prop_emoji = "❓️";
            let mut prop_value = String::new();

            match &prop.value {
                NotionPropertyValue::Date(Some(value)) => {
                    date = Date::try_from(value.clone()).ok();
                }
                NotionPropertyValue::Number(value) => {
                    prop_emoji = "🔢";
                    prop_value = value.unwrap_or_default().to_string();
                }
                NotionPropertyValue::RichText(value) => {
                    let mut text = String::new();
                    for t in value {
                        text += &format!("{} ", t.plain_text);
                    }

                    prop_emoji = "🇹";
                    prop_value = text;
                }
                NotionPropertyValue::Url(value) => {
                    prop_emoji = "ℹ️";
                    prop_value = value.clone().unwrap_or_default();
                }
                NotionPropertyValue::MultiSelect(value) => {
                    let mut text = String::new();
                    for ms in value {
                        text += &format!("{} ", ms.name);
                    }

                    prop_emoji = "➡️";
                    prop_value = text;
                }
                NotionPropertyValue::Select(Some(value)) => {
                    prop_emoji = "▶️";
                    prop_value = value.name.clone();
                }
                NotionPropertyValue::People(value) => {
                    let mut text = String::new();
                    for ms in value {
                        text += &format!("{} ", ms.name.clone().unwrap_or("NoName".into()));
                    }

                    prop_emoji = "🚹";
                    prop_value = text;
                }
                NotionPropertyValue::Status(value) => {
                    prop_emoji = "⏺️";
                    prop_value = value.name.clone();

                    title_prefix = match &value.id {
                        NotionSelectType::Done => "✅".into(),
                        NotionSelectType::InProgress => "🟧".into(),
                        NotionSelectType::NotStarted => "🔲".into(),
                        _ => "".into(),
                    };
                }
                NotionPropertyValue::Relation(value) => {
                    for relation in value {
                        println!("Fetching database relation... ({prop_name})");
                        let page = fetch_notion_page(client, &relation.id).await;
                        match page {
                            Ok(page) => {
                                prop_emoji = "↗️";
                                prop_value = page.title();
                            }
                            Err(err) => {
                                println!("Unable to retrieve Notion relation: {}", err.to_string())
                            }
                        }
                    }
                }
                _ => (),
            }

            if !prop_value.is_empty() {
                additional.insert(format!("{prop_emoji} {prop_name}"), prop_value);
            }
        }

        let mut title = database_page.title();
        if !title_prefix.is_empty() {
            title = format!("({title_prefix}) {title}");
        }

        // We only care about pages which have a `Date` property set
        let last_edited = DateTime::parse_from_rfc3339(&database_page.last_edited_time)
            .expect("Unable to parse Notion last_edited_time");

        if let Some(date) = date {
            let entry = DateEntry {
                id: database_page.id.clone(),
                title,
                date,
                last_edited,
                url: database_page.url.clone(),
                additional,
            };
            date_entries.push(entry);
        }
    }

    date_entries
}

async fn fetch_notion_page(
    client: &Client,
    page_id: &str,
) -> Result<NotionPage, Error<serde_json::Error>> {
    let db_uri = format!("pages/{page_id}");
    let mut res = client.get(db_uri).await.unwrap();
    let content = res.body_string().await.unwrap();

    let deserializer = &mut serde_json::Deserializer::from_str(&content);
    serde_path_to_error::deserialize(deserializer)
}

/// Write *.ics file
fn create_ics(entries: &Vec<DateEntry>, path: &str) {
    let mut calendar = Calendar::new().name("notion2ics").done();

    for entry in entries {
        let event = entry.as_event();
        calendar.push(event);
    }

    // Write file
    println!("Writing file to: {path}");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();

    file.write_all(calendar.to_string().as_bytes()).unwrap();
    file.flush().unwrap();
}
