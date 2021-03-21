use configparser;
use chrono::prelude::*;
use clap;
use dirs;
use read_input::prelude::*;

/*
IMPROVEMENT IDEAS:
    + alert when adding duplicate game name
    + case-insentive game names
    + parse steam/dekudeals
    + disown
    + provide fixed list of ownership types, pick rather than free string
    + abandon changes without crashing during each subcommand
*/

struct SGameRecord {
    id: u32,
    title: String,
    release_year: u16,
    via: String,
    play_more: bool,
    passes: u16,
    next_valid_date: Date<Utc>,

    eternal: Option<bool>,
    linux: Option<bool>,
    couch: Option<bool>,
}

impl SGameRecord {
    fn new(column_map: &SGameRecordColumnMap, csv_record: &csv::StringRecord) -> Result<Self, Box<dyn std::error::Error>> {
        // -- unwrap here because we know how many columns our CSV has from the column_map
        Ok(Self{
            id: csv_record.get(column_map.id_column).unwrap().parse::<u32>()?,
            title: String::from(csv_record.get(column_map.title_column).unwrap()),
            release_year: csv_record.get(column_map.release_year_column).unwrap().parse::<u16>()?,
            via: String::from(csv_record.get(column_map.via_column).unwrap()),
            play_more: csv_record.get(column_map.play_more_column).unwrap().parse::<u16>()? > 0,
            passes: csv_record.get(column_map.passes_column).unwrap().parse::<u16>()?,
            // $$$FRK(TODO): parse date
            next_valid_date: panic!(),
            // $$$FRK(TODO): use Option.map() here to do the parse and comparison
            eternal: csv_record.get(column_map.eternal_column).unwrap().parse::<u16>()? > 0,
            linux: csv_record.get(column_map.linux_column).unwrap().parse::<u16>()? > 0,
            couch: csv_record.get(column_map.couch_column).unwrap().parse::<u16>()? > 0,
        })
    }
}

struct SGameRecordColumnMap {
    id_column: usize,
    title_column: usize,
    release_year_column: usize,
    via_column: usize,
    play_more_column: usize,
    passes_column: usize,
    next_valid_date_column: usize,
    eternal_column: usize,
    linux_column: usize,
    couch_column: usize,
}

impl SGameRecordColumnMap {
    fn new<R: std::io::Read>(reader: &mut csv::Reader<R>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut id_column = None;
        let mut title_column = None;
        let mut release_year_column = None;
        let mut via_column = None;
        let mut play_more_column = None;
        let mut passes_column = None;
        let mut next_valid_date_column = None;
        let mut eternal_column = None;
        let mut linux_column = None;
        let mut couch_column = None;

        for (i, header) in reader.headers()?.iter().enumerate() {
            match header {
                "id" => id_column = Some(i),
                "title" => title_column = Some(i),
                "release_year" => release_year_column = Some(i),
                "via" => via_column = Some(i),
                "play_more" => play_more = Some(i),
                "passes" => passes_column = Some(i),
                "next_valid_date" => next_valid_date_column = Some(i),
                "external" => eternal_column = Some(i),
                "linux" => linux_column = Some(i),
                "couch" => couch_column = Some(i),
                _ => (),
            }
        }

        Ok(Self {
            id_column: id_column.ok_or("_game.csv is missing column 'id'")?,
            title_column: title_column.ok_or("_game.csv is missing column 'title'")?,
            release_year_column: release_year_column.ok_or("_game.csv is missing column 'release_year'")?,
            via_column: via_column.ok_or("_game.csv is missing column 'via'")?,
            play_more_column: play_more_column.ok_or("_game.csv is missing column 'play_more'")?,
            passes_column: passes_column.ok_or("_game.csv is missing column 'passes'")?,
            next_valid_date_column: next_valid_date_column.ok_or("_game.csv is missing column 'next_valid_date'")?,
            eternal_column: eternal_column.ok_or("_game.csv is missing column 'eternal'")?,
            linux_column: linux_column.ok_or("_game.csv is missing column 'linux'")?,
            couch_column: couch_column.ok_or("_game.csv is missing column 'couch'")?,
        })
    }
}

struct SOwnRecord {
    game_id: u32,
    own_type: String,
}

struct SSessionRecord {
    game_id: u32,
    started_date: Date<Utc>,
    finished_date: Date<Utc>,
    notable: Option<bool>,
}

struct SDB {
    games: Vec<SGameRecord>,
    ownership: Vec<SOwnRecord>,
    sessions: Vec<SSessionRecord>,

    next_id: u32,
}

impl SDB {
    fn load(data_directory: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path_buf = std::path::PathBuf::new();
        path_buf.push(data_directory);
        path_buf.push("_game.csv");
        println!("game csv: {:?}", path_buf);

        let mut games_reader = csv::Reader::from_path(path_buf).unwrap();
        let games_column_map = SGameRecordColumnMap::new(&mut games_reader);
        for result in games_reader.records() {
            for (i, entry) in result?.iter().enumerate() {
                if games_column_map.id_column == i {

                }
            }
        }

        panic!();
    }
}

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

fn handle_add(data_directory: String, cli_matches: &clap::ArgMatches) {
    let _ = SDB::load(data_directory).unwrap();

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
        handle_add(data_directory, add_matches);
    }
}
