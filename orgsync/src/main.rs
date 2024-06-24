use chrono::{DateTime, Duration, Utc};
use orgize::{elements::Timestamp, Org};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
struct EventDateTime {
    date_time: DateTime<Utc>,
    time_zone: String,
}

#[derive(Serialize)]
struct Event {
    summary: String,
    start: EventDateTime,
    end: EventDateTime,
}

#[derive(Deserialize)]
struct CalendarApiToken {
    access_token: String,
}

async fn authenticate_google_calendar(token_file: &str) -> String {
    let token_json = fs::read_to_string(token_file).expect("Unable to read the token file");
    let token: CalendarApiToken =
        serde_json::from_str(&token_json).expect("Unable to parse the token JSON");
    token.access_token
}

async fn sync_to_google_calendar(
    tasks: Vec<(String, DateTime<Utc>)>,
    token: String,
    calendar_id: &str,
) {
    let client = Client::new();

    for (task, deadline) in tasks {
        let event = Event {
            summary: task,
            start: EventDateTime {
                date_time: deadline,
                time_zone: "UTC".to_string(),
            },
            end: EventDateTime {
                date_time: deadline + Duration::hours(1),
                time_zone: "UTC".to_string(),
            },
        };

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            calendar_id
        );
        let response = client
            .post(&url)
            .bearer_auth(token.clone())
            .json(&event)
            .send()
            .await
            .expect("Unable to create an event");

        if response.status().is_success() {
            println!("Event created: {:?}", event.summary);
        } else {
            eprintln!(
                "Failed to create event: {:?}",
                response.text().await.unwrap()
            );
        }
    }
}

fn read_org_mode_tasks(org_file: &str) -> Vec<(String, DateTime<Utc>)> {
    let content = fs::read_to_string(org_file).expect("Unable to read the org-mode file");
    let org = Org::parse(&content);
    let mut tasks = vec![];

    for headline in org.headlines() {
        let title = headline.title(&org);
        if let Some(deadline) = title.deadline() {
            match deadline {
                Timestamp::Active {
                    start,
                    repeater: _,
                    delay: _,
                } => {
                    tasks.push((String::from(title.raw.clone()), start.into()));
                }
                _ => {}
            }
        }
    }

    tasks
}

#[tokio::main]
async fn main() {
    let org_file = "path/to/your/org-mode-file.org";
    let google_credentials_file = "path/to/your/google_credentials.json";
    let calendar_id = "primary";

    let tasks = read_org_mode_tasks(org_file);

    let access_token = authenticate_google_calendar(google_credentials_file).await;
    sync_to_google_calendar(tasks, access_token, calendar_id).await;
}

