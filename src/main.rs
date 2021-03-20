use configparser;
use clap;
use dirs;
use read_input::prelude::*;

/*
IMPROVEMENT IDEAS:
    + alert when adding duplicate game name
    + case-insentive game names
    + parse steam/dekudeals
*/

fn parse_data_directory() -> String {
    let mut config_path = dirs::home_dir().expect("Unable to resolve home directory for user.");
    config_path.push(".gamechooser");

    if !config_path.exists() {
        eprintln!("Please create config file ~/.gamechooser to specify where your game database will be stored.");
        std::process::exit(1);
    }

    let config_str = match std::fs::read_to_string(config_path) {
        Err(e) => {
            eprintln!("Failed to read config file with error {:?}", e);
            std::process::exit(1);
        }
        Ok(s) => {
            s
        }
    };

    let ini = {
        let mut temp = configparser::ini::Ini::new();
        match temp.read(config_str) {
            Err(e) => {
                eprintln!("Failed to create config parser with error {:?}", e);
                std::process::exit(1);
            }
            Ok(_) => temp,
        }
    };

    match ini.get("main", "data_directory") {
        None => {
            eprintln!("Config file is missing 'data_directory' value in 'main' section");
            std::process::exit(1);
        }
        Some(data_directory) => {
            data_directory
        }
    }
}

fn handle_add(cli_matches: &clap::ArgMatches) {
    let title : String = input().msg("Game title: ").get();

    println!("Title: {:?}", title);
}

fn main() {
    let data_directory = parse_data_directory();

    use clap::{App, SubCommand, Arg};

    let matches = App::new("gamechooser")
        .about("Maintain a library of video games and randomly select what to play.")
        .subcommand(SubCommand::with_name("sessions")
            .about("List sessions of games.")
            .arg(Arg::with_name("inactive")
                .short("i")
                .long("inactive")
                .help("Show inactive sessions"))
            .arg(Arg::with_name("year")
                .short("y")
                .long("year")
                .help("Limits sessions to a specific year. By default, current year. 0 for all years.")
                .takes_value(true))
        )
        .subcommand(SubCommand::with_name("add")
            .about("Add a new game to database.")
        )
        .subcommand(SubCommand::with_name("gamestats")
            .about("Print stats on the database state.")
        )
        .subcommand(SubCommand::with_name("start")
            .about("Start a session.")
            .arg(Arg::with_name("title")
                .help("Title of game to start a session for.")
                .required(true)
                .takes_value(true))
        )
        .subcommand(SubCommand::with_name("finish")
            .about("Finish a game session.")
        )
        .subcommand(SubCommand::with_name("search")
            .about("Search for a game.")
            .arg(Arg::with_name("title")
                .help("Title of game to search for.")
                .required(true)
                .takes_value(true))
        )
        .subcommand(SubCommand::with_name("own")
            .about("Add ownership record for a game")
            .arg(Arg::with_name("title")
                .help("Title of game to add record for.")
                .required(true)
                .takes_value(true))
        )
        .subcommand(SubCommand::with_name("select")
            .about("Select a random game to play.")
            .arg(Arg::with_name("no_session")
                .help("Don't start a session after picking a game.")
                .long("ns")
            )
            .arg(Arg::with_name("couch")
                .help("Only select games playable on the couch.")
                .short("c")
                .long("couch")
            )
            .arg(Arg::with_name("portable")
                .help("Only select games playable portably.")
                .short("p")
                .long("portable")
            )
            .arg(Arg::with_name("buy")
                .help("Include games that are not owned.")
                .short("b")
                .long("buy")
            )
            .arg(Arg::with_name("num")
                .help("Number of games to show each round.")
                .short("n")
                .long("num")
                .takes_value(true)
            )
            .arg(Arg::with_name("max_passes")
                .help("Show only games passed up to this many times (default 2).")
                .short("m")
                .long("max_passes")
                .takes_value(true)
            )
        )
        .subcommand(SubCommand::with_name("reset")
            .about("Reset a game to 0 passes.")
            .arg(Arg::with_name("title")
                .help("Title of game to reset.")
                .required(true)
            )
        )
        .get_matches();

    if let Some(add_matches) = matches.subcommand_matches("add") {
        handle_add(add_matches);
    }
}
