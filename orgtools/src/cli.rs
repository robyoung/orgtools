use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command};

fn parse_keyword(s: &str) -> Result<String, String> {
    Ok(s.trim().to_uppercase())
}

pub fn cli() -> Cli {
    let matches = create_command().get_matches();
    Cli::from_matches(&matches)
}

fn create_command() -> Command {
    let input_file = Arg::new("input_file")
        .help("Input file path")
        .required(false);
    let output_file = Arg::new("output_file")
        .long("output-file")
        .help("Output file path")
        .required(false);

    Command::new("orgtools")
        .about("A tool for managing org files")
        .subcommand_required(true)
        .arg(
            Arg::new("keywords_unfinished")
                .long("keywords-unfinished")
                .value_parser(parse_keyword)
                .value_delimiter(',')
                .default_value("TODO,DOING,BLOCKED")
                .help("Keywords for unfinished tasks"),
        )
        .arg(
            Arg::new("keywords_finished")
                .long("keywords-finished")
                .value_parser(parse_keyword)
                .value_delimiter(',')
                .default_value("DONE,ABANDONED")
                .help("Keywords for finished tasks"),
        )
        .subcommand(
            Command::new("prune")
                .about("Remove finished tasks")
                .arg(input_file.clone())
                .arg(output_file.clone()),
        )
        .subcommand(
            Command::new("tree")
                .about("Display tree structure")
                .arg(input_file.clone())
                .arg(
                    Arg::new("sexp")
                        .long("sexp")
                        .action(ArgAction::SetTrue)
                        .help("Display tree in S-expression format"),
                )
                .arg(
                    Arg::new("sections")
                        .long("sections")
                        .action(ArgAction::SetTrue)
                        .help("Display sections"),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("List tasks")
                .arg(input_file.clone()),
        )
        .subcommand(
            Command::new("add")
                .about("Add a task")
                .arg(input_file.clone())
                .arg(output_file.clone())
                .arg(
                    Arg::new("under")
                        .long("under")
                        .help("Headline under which to add the task"),
                )
                .arg(
                    Arg::new("after")
                        .long("after")
                        .help("Headline after which to add the task"),
                )
                .group(
                    ArgGroup::new("under_or_after")
                        .args(["under", "after"])
                        .required(false)
                        .multiple(false),
                ),
        )
}

#[derive(Debug)]
pub struct Cli {
    pub keywords_unfinished: Vec<String>,
    pub keywords_finished: Vec<String>,
    pub command: Commands,
}

impl Cli {
    pub fn config(&self) -> Config {
        Config {
            keywords_unfinished: self.keywords_unfinished.clone(),
            keywords_finished: self.keywords_finished.clone(),
        }
    }

    fn from_matches(matches: &ArgMatches) -> Self {
        let keywords_unfinished = matches
            .get_many::<String>("keywords_unfinished")
            .unwrap()
            .cloned()
            .collect();
        let keywords_finished = matches
            .get_many::<String>("keywords_finished")
            .unwrap()
            .cloned()
            .collect();

        let command = match matches.subcommand() {
            Some(("prune", sub_matches)) => Commands::Prune {
                input_file: sub_matches.get_one::<String>("input_file").cloned(),
                output_file: sub_matches.get_one::<String>("output_file").cloned(),
            },
            Some(("tree", sub_matches)) => Commands::Tree {
                input_file: sub_matches.get_one::<String>("input_file").cloned(),
                sexp: sub_matches.get_flag("sexp"),
                sections: sub_matches.get_flag("sections"),
            },
            Some(("list", sub_matches)) => Commands::List {
                input_file: sub_matches.get_one::<String>("input_file").cloned(),
            },
            Some(("add", sub_matches)) => Commands::Add {
                input_file: sub_matches.get_one::<String>("input_file").cloned(),
                output_file: sub_matches.get_one::<String>("output_file").cloned(),
                headline: sub_matches.get_one::<String>("headline").unwrap().clone(),
                under: sub_matches.get_one::<String>("under").cloned(),
                after: sub_matches.get_one::<String>("after").cloned(),
            },
            _ => unreachable!(),
        };

        Cli {
            keywords_unfinished,
            keywords_finished,
            command,
        }
    }
}

#[derive(Debug)]
pub enum Commands {
    Prune {
        input_file: Option<String>,
        output_file: Option<String>,
    },
    Tree {
        input_file: Option<String>,
        sexp: bool,
        sections: bool,
    },
    List {
        input_file: Option<String>,
    },
    Add {
        input_file: Option<String>,
        output_file: Option<String>,
        headline: String,
        under: Option<String>,
        after: Option<String>,
    },
}

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
