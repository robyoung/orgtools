use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Form, Router,
};
use orgtools::org::{Org, OrgFile};
use serde::Deserialize;
use tokio::fs;

#[derive(Clone)]
struct AppState {
    config: Config,
}

#[derive(Deserialize)]
struct JournalEntry {
    content: String,
}

#[derive(Clone)]
struct Config {
    filename: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            filename: std::env::var("JOURNAL_FILE").unwrap_or_else(|_| "journal.org".to_string()),
        }
    }

    async fn read_org_file(&self) -> String {
        fs::read_to_string(&self.filename)
            .await
            .expect("Org file not readable")
    }

    async fn write_org_file(&self, content: &str) {
        fs::write(&self.filename, content)
            .await
            .expect("Org file not writable");
    }

    fn org<'a>(&self, input: &'a str) -> OrgFile<'a> {
        Org::new().load(input)
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    let app_state = AppState { config };

    let app = Router::new()
        .route("/", get(index))
        .route("/submit", post(submit_entry))
        .with_state(app_state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let org_content = state.config.read_org_file().await;
    let org = state.config.org(&org_content);

    let mut entries = org
        .subsections()
        .into_iter()
        .filter_map(|section| {
            let headline = section.headline_text()?;
            let body = section.body_text()?;
            // TODO: Render this with a template engine
            // TODO: Render as markdown
            // TODO: Protect against XSS
            Some(format!("<h2>{}</h2>\n<p>{}</p>", headline, body))
        })
        .collect::<Vec<_>>();
    entries.reverse(); // Most recent at the top

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Journal</title></head>
        <body>
            <form action="/submit" method="post">
                <textarea name="content" rows="10" cols="50"></textarea><br>
                <input type="submit" value="Submit Entry">
            </form>
            <hr>
            {}
        </body>
        </html>
        "#,
        entries.join("<hr>")
    ))
}

async fn submit_entry(
    State(state): State<AppState>,
    Form(entry): Form<JournalEntry>,
) -> axum::response::Redirect {
    let org_content = state.config.read_org_file().await;
    let output = {
        let org = state.config.org(&org_content);

        let mut output_builder = org.output_builder();

        let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        let section = output_builder
            .new_section()
            .headline(&date)
            .body(&entry.content.trim())
            .render();

        let mut output = output_builder.append_to_end_of_input();
        output.push_str(&section);
        output
    };

    state.config.write_org_file(&output).await;

    axum::response::Redirect::to("/")
}
