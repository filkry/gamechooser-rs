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

type DynError = Box<dyn std::error::Error>;

#[derive(Debug)]
struct SGameRecord {
    id: u32,
    title: String,
    release_year: u16,
    via: String,
    play_more: bool,
    passes: u16,
    next_valid_date: Option<NaiveDate>,

    eternal: Option<bool>,
    linux: Option<bool>,
    couch: Option<bool>,
}

fn parse_bool_int(s: &str) -> Option<bool> {
    s.parse::<u16>().ok().map(|intval| intval > 0)
}

impl SGameRecord {
    fn new(column_map: &SGameRecordColumnMap, csv_record: csv::StringRecord) -> Result<Self, DynError> {
        // -- unwrap here because we know how many columns our CSV has from the column_map
        Ok(Self{
            id: csv_record.get(column_map.id_column).unwrap().parse::<u32>()?,
            title: String::from(csv_record.get(column_map.title_column).unwrap()),
            release_year: csv_record.get(column_map.release_year_column).unwrap().parse::<u16>()?,
            via: String::from(csv_record.get(column_map.via_column).unwrap()),
            play_more: csv_record.get(column_map.play_more_column).unwrap().parse::<u16>()? > 0,
            passes: csv_record.get(column_map.passes_column).unwrap().parse::<u16>()?,
            next_valid_date: NaiveDate::parse_from_str(csv_record.get(column_map.next_valid_date_column).unwrap(), "%Y-%m-%d").ok(),
            eternal: parse_bool_int(csv_record.get(column_map.eternal_column).unwrap()),
            linux: parse_bool_int(csv_record.get(column_map.linux_column).unwrap()),
            couch: parse_bool_int(csv_record.get(column_map.couch_column).unwrap()),
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
    fn new<R: std::io::Read>(reader: &mut csv::Reader<R>) -> Result<Self, DynError> {
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
                "play_more" => play_more_column = Some(i),
                "passes" => passes_column = Some(i),
                "next_valid_date" => next_valid_date_column = Some(i),
                "eternal" => eternal_column = Some(i),
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

impl SOwnRecord {
    fn new(column_map: &SOwnRecordColumnMap, csv_record: csv::StringRecord) -> Result<Self, DynError> {
        // -- unwrap here because we know how many columns our CSV has from the column_map
        Ok(Self{
            game_id: csv_record.get(column_map.game_id_column).unwrap().parse::<u32>()?,
            own_type: String::from(csv_record.get(column_map.own_type_column).unwrap()),
        })
    }
}

struct SOwnRecordColumnMap {
    game_id_column: usize,
    own_type_column: usize,
}

impl SOwnRecordColumnMap {
    fn new<R: std::io::Read>(reader: &mut csv::Reader<R>) -> Result<Self, DynError> {
        let mut game_id_column = None;
        let mut own_type_column = None;

        for (i, header) in reader.headers()?.iter().enumerate() {
            match header {
                "game_id" => game_id_column = Some(i),
                "storefront" => own_type_column = Some(i), // legacy support
                "own_type" => own_type_column = Some(i),
                _ => (),
            }
        }

        Ok(Self {
            game_id_column: game_id_column.ok_or("_game.csv is missing column 'game_id'")?,
            own_type_column: own_type_column.ok_or("_game.csv is missing column 'own_type' or 'storefront'")?,
        })
    }
}

struct SSessionRecord {
    game_id: u32,
    started_date: Option<NaiveDate>,
    finished_date: Option<NaiveDate>,
    notable: Option<bool>,
}

fn parse_notable(s: &str) -> Option<bool> {
    match s {
        "transient" => Some(false),
        "stuck" => Some(true),
        _ => {
            match s.parse::<u16>() {
                Ok(intval) => Some(intval > 0),
                Err(_) => None,
            }
        },
    }
}

impl SSessionRecord {
    fn new(column_map: &SSessionRecordColumnMap, csv_record: csv::StringRecord) -> Result<Self, DynError> {
        // -- unwrap here because we know how many columns our CSV has from the column_map
        Ok(Self{
            game_id: csv_record.get(column_map.game_id_column).unwrap().parse::<u32>()?,
            started_date: NaiveDate::parse_from_str(csv_record.get(column_map.started_date_column).unwrap(), "%Y-%m-%d").ok(),
            finished_date: column_map.finished_date_column.and_then(|column| {
                NaiveDate::parse_from_str(csv_record.get(column).unwrap(), "%Y-%m-%d").ok()
            }),
            notable: parse_notable(csv_record.get(column_map.notable_column).unwrap()),
        })
    }
}

struct SSessionRecordColumnMap {
    game_id_column: usize,
    started_date_column: usize,
    finished_date_column: Option<usize>,
    notable_column: usize,
}

impl SSessionRecordColumnMap {
    fn new<R: std::io::Read>(reader: &mut csv::Reader<R>) -> Result<Self, DynError> {
        let mut game_id_column = None;
        let mut started_date_column = None;
        let mut finished_date_column = None;
        let mut notable_column = None;

        for (i, header) in reader.headers()?.iter().enumerate() {
            match header {
                "game_id" => game_id_column = Some(i),
                "started" => started_date_column = Some(i), // legacy support
                "started_date" => started_date_column = Some(i),
                "finished_date" => finished_date_column = Some(i),
                "outcome" => notable_column = Some(i), // legacy support
                "notable" => notable_column = Some(i),
                _ => (),
            }
        }

        Ok(Self {
            game_id_column: game_id_column.ok_or("_game.csv is missing column 'game_id'")?,
            started_date_column: started_date_column.ok_or("_game.csv is missing column 'started_date' or 'started'")?,
            finished_date_column,
            notable_column: notable_column.ok_or("_game.csv is missing column 'notable' or 'outcome'")?,
        })
    }
}

struct SDB {
    games: Vec<SGameRecord>,
    ownership: Vec<SOwnRecord>,
    sessions: Vec<SSessionRecord>,

    next_id: u32,
}

impl SDB {
    fn load(data_directory: String) -> Result<Self, DynError> {
        let mut games : Vec<SGameRecord> = Vec::new();
        let mut ownership : Vec<SOwnRecord> = Vec::new();
        let mut sessions : Vec<SSessionRecord> = Vec::new();

        {
            let mut path_buf = std::path::PathBuf::new();
            path_buf.push(&data_directory);
            path_buf.push("_game.csv");

            let mut games_reader = csv::Reader::from_path(path_buf).unwrap();
            let games_column_map = SGameRecordColumnMap::new(&mut games_reader)?;
            for result in games_reader.records() {
                games.push(SGameRecord::new(&games_column_map, result?)?);
            }
        }

        {
            let mut path_buf = std::path::PathBuf::new();
            path_buf.push(&data_directory);
            path_buf.push("_own.csv");

            let mut own_reader = csv::Reader::from_path(path_buf).unwrap();
            let own_column_map = SOwnRecordColumnMap::new(&mut own_reader)?;
            for result in own_reader.records() {
                ownership.push(SOwnRecord::new(&own_column_map, result?)?);
            }
        }

        {
            let mut path_buf = std::path::PathBuf::new();
            path_buf.push(&data_directory);
            path_buf.push("_session.csv");

            let mut session_reader = csv::Reader::from_path(path_buf).unwrap();
            let session_column_map = SSessionRecordColumnMap::new(&mut session_reader)?;
            for result in session_reader.records() {
                sessions.push(SSessionRecord::new(&session_column_map, result?)?);
            }
        }

        let mut next_id = 0;
        for game in &games {
            if game.id > next_id {
                next_id = game.id + 1;
            }
        }

        Ok(Self {
            games,
            ownership,
            sessions,
            next_id,
        })
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
    let dberr = SDB::load(data_directory);
    if let Err(e) = dberr {
        eprintln!("Unable to load database, error message '{:?}'", e);
        return;
    }

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
