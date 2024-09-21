#[derive(Debug, Clone)]
pub struct Config {
    pub keywords_unfinished: Vec<String>,
    pub keywords_finished: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            keywords_unfinished: vec![
                "TODO".to_string(),
                "DOING".to_string(),
                "BLOCKED".to_string(),
            ],
            keywords_finished: vec!["DONE".to_string(), "ABANDONED".to_string()],
        }
    }
}
