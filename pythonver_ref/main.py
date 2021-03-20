import argparse, csv, sqlite3, db, os, sys
from datetime import date
import datetime
import record_print as rp
import configparser as cp

data_directory = None

def instantiate_db(load_csvs = False):
    conn = sqlite3.connect(':memory:')
    conn.row_factory = sqlite3.Row
    db.create_schema(conn)
    path = os.path.expanduser(data_directory)

    if load_csvs:
        db.load_csvs(conn, path)

    return conn, path

def annotate_platforms(conn, game_records):
    for gr in game_records:
        platforms = db.storefronts(conn, gr['id'])
        gr['platforms'] = ', '.join(platforms)

def handle_import(args):
    conn, path =  instantiate_db(False)

    # TODO: confirmation step? This wipes out existing data
    with open(args.file_name, 'r') as csvfile:
        db.import_gdoc_games(conn, csv.reader(csvfile))

    if args.sessions:
        with open(args.sessions, 'r') as csvfile:
            db.import_gdoc_sessions(conn, csv.reader(csvfile))

    db.dump_csvs(conn, path)

def handle_add(args):
    conn, path = instantiate_db(True)

    title = input("Game title: ")
    year = input("Release year: ")
    year = int(year) if len(year) > 0 else None
    linux = True if input("Linux (y/n): ").lower() == 'y' else False
    couch = True if input("Couch playable (y/n): ").lower() == 'y' else False
    portable = True if input("Portable (y/n): ").lower() == 'y' else False
    via = input("Via: ")
    owned_on = input("Owned platforms (comma separated): ").split(',')

    db.add_game(conn, title, year, linux, True, couch, portable, 0, via, None, owned_on)

    db.dump_csvs(conn, path)

def handle_starts(args):
    conn, path = instantiate_db(True)

    games = db.search_game(conn, args.title)[:5]

    print('Which game?')
    print(rp.format_records(games,
        ['title'],
        header = True, nums = True))

    which_game = input('Input number, or q to abort: ')
    if which_game.lower() == 'q':
        return

    gid, title = (games[int(which_game) - 1]['id'],
                  games[int(which_game) - 1]['title'])
    db.create_session(conn, gid)

    print('Created session for %s' % (title))

    db.dump_csvs(conn, path)

def handle_own(args):
    conn, path = instantiate_db(True)

    games = db.search_game(conn, args.title)[:5]

    print('Which game now owned?')
    print(rp.format_records(games,
        ['title'],
        nums = True,header = True))

    which_game = input('Input number, or q to abort: ')
    if which_game.lower() == 'q':
        return

    gid, title = (games[int(which_game) - 1]['id'],
                  games[int(which_game) - 1]['title'])

    plat = input('Name of platformed purchased on: ')
    db.add_ownership(conn, gid, plat)

    print('Added %s platform for %s' % (plat, title))

    db.dump_csvs(conn, path)

def handle_search(args):
    conn, path = instantiate_db(True)

    games = db.search_game(conn, args.title)[:5]
    annotate_platforms(conn, games)
    columns = ['title', 'via', 'platforms', 'release_year']

    if args.passes:
        columns.append('passes')
    if args.eternal:
        columns.append('eternal')

    print(rp.format_records(games, columns, header=True))

def handle_reset(args):
    conn, path = instantiate_db(True)

    games = db.search_game(conn, args.title)[:5]

    print('Which game to reset?')
    print(rp.format_records(games,
        ['title'],
        nums = True,header = True))

    which_game = input('Input number, or q to abort: ')
    if which_game.lower() == 'q':
        return

    gid, title = (games[int(which_game) - 1]['id'],
                  games[int(which_game) - 1]['title'])

    db.reset_selectability(conn, gid);
    print('Reset selectability for %s' % (title))

    db.dump_csvs(conn, path)

def handle_gamestats(args):
    conn, path = instantiate_db(True)

    all_games = db.select_random_games(conn, n=0, owned=None)
    n_all_games = len(all_games)
    owned_games = db.select_random_games(conn, n=0, owned=True)
    n_owned_games = len(owned_games)

    print('selectable games: ', str(n_all_games))
    print('owned selectable games: ', str(n_owned_games))
    print('unowned selectable games: ', str(n_all_games - n_owned_games))


def handle_select(args):
    conn, path = instantiate_db(True)

    passed_ids = []
    while True:
        owned = True
        if args.buy:
            owned = None
        if args.buy_only:
            owned = False

        games = db.select_random_games(conn, n = args.n, before_this_year = True if args.old else None,
                linux = True if args.linux else None, couch = True if args.couch else None, portable = True if args.portable else None,
                owned = owned, max_passes = args.max_passes,
                exclude_ids = passed_ids, storefront = args.storefront)

        annotate_platforms(conn, games)
        print("")
        print(rp.format_records(games,
            ['title', 'linux', 'couch', 'portable', 'platforms', 'via'],
            header = True, nums = True))

        # If we're just displaying a selection, finish here
        if not args.pick:
            break

        print('\nChoose a game to create a new active session for. Input 0 to pass on all games. -1 to push timer without passing (i.e. too expensive). Q to abort.')
        selection = input("Selection: ")

        if selection == 'q' or selection == 'Q':
            break

        selection = int(selection)

        if selection == 0:
            # Increment the pass counter on each game
            for game in games:
                # Don't propose game again
                passed_ids.append(game['id'])

                # If eternal still undecided
                # give option to make eternal
                if game['eternal'] is None:
                    eternal = input('Should this game never stop being proposed? Y/N/P[ass]: ')
                    if eternal == 'Y' or eternal == 'y':
                        db.set_eternal(conn, game['id'], 1)
                    elif eternal  == 'N' or eternal == 'n':
                        db.set_eternal(conn, game['id'], 0)


                # If the game is not out yet, don't increment
                if game['release_year'] != None and game['release_year'] != '' and int(game['release_year']) == date.today().year:
                    freebie = input('%s was released this year. Has it been released? Y/N: ' % game['title'])
                    if freebie == 'N' or freebie == 'n':
                        continue
                new_passes = db.inc_pass(conn, game['id'])

                # Delay next possible proposal according to passes
                if new_passes == 1:
                    db.set_next_valid_date(conn, game['id'],
                        date.today() + datetime.timedelta(days = 30))
                elif new_passes == 2:
                    db.set_next_valid_date(conn, game['id'],
                        date.today() + datetime.timedelta(days = 90))
                else:
                    db.set_next_valid_date(conn, game['id'],
                        date.today() + datetime.timedelta(days = 180))

        elif selection == -1:
            for game in games:
                db.set_next_valid_date(conn, game['id'],
                    date.today() + datetime.timedelta(days = 90))

            continue
        else:
            # Create an active session
            game = games[selection - 1]
            db.create_session(conn, game['id'])
            print('Created a new session of %s.' % game['title'])
            break

        print('\n')

    # So scared of commitment
    db.dump_csvs(conn, path)

def handle_sessions(args):
    conn, path = instantiate_db(True)

    session_records = db.show_sessions(conn, active = not args.inactive,
            session_year = None if args.year == 0 else int(args.year),
            status = 'stuck' if args.stuck else None)

    columns = ['title', 'started']
    if(args.column):
        columns.extend(args.column)
    print(rp.format_records(session_records, columns,
        header=True,nums=True))

def handle_finish(conn):
    conn, path = instantiate_db(True)

    sessions = db.show_sessions(conn, active = True)
    print(rp.format_records(sessions,
        ['title', 'started'],
        header=True,nums=True))

    finish = input('Input a session to finish, or Q to abort: ')
    if finish == 'q' or finish == 'Q':
        return
    finish = int(finish)

    finish_session = sessions[finish - 1]
    status = input('Was %s a transient (t) or sticking (s) experience: ' % finish_session['title'])
    if status == 'q' or status == 'Q':
        return

    status = 'transient' if status == 't' or status == 'T' else 'stuck'

    # TODO: allow delays
    more = input('''How long until %s should be suggested again?
1) any time
2) one month
3) three months
4) one year
5) done forever
q) abort
Input response: ''' % finish_session['title'])
    if more == 'q' or more == 'Q':
        return

    if int(more) == 2:
        db.set_next_valid_date(conn, finish_session['game_id'],
            date.today() + datetime.timedelta(days = 31))
    elif int(more) == 3:
        db.set_next_valid_date(conn, finish_session['game_id'],
            date.today() + datetime.timedelta(days = 92))
    elif int(more) == 4:
        db.set_next_valid_date(conn, finish_session['game_id'],
            date.today() + datetime.timedelta(days = 365))
    elif int(more) == 5:
        db.retire_game(conn, finish_session['game_id'])

    db.finish_session(conn, finish_session['game_id'], status)

    db.dump_csvs(conn, path)


if __name__ == '__main__':
    configpath = os.path.expanduser("~/.gamechooser")
    if not os.path.exists(configpath):
        print("Please create config file ~/.gamechooser to specify where your game database will be stored.")
        sys.exit(1)

    config = cp.RawConfigParser()
    config.read(configpath)
    data_directory = config.get("main", "data_directory")

    if data_directory is None:
        print("Please ensure 'data_directory' is specified in the 'main' section of your config file before running this software.")

    arg_parser = argparse.ArgumentParser(description="Manage a library of video games.")
    subparsers = arg_parser.add_subparsers()

    # Parameteres for listing sessions
    sessions_parser = subparsers.add_parser('sessions', help='List sessions of games.')
    sessions_parser.add_argument('-i', '--inactive', help='Show inactive sessions.',
            action='store_true')
    sessions_parser.add_argument('-y', '--year',
            help='Limit sessions to a specific year. By default, current year. 0 for all years.',
            action='store', default=date.today().year)
    sessions_parser.add_argument('-s', '--stuck',
            help='Show only sessions which "stuck" (made an impression).',
            action='store_true')
    sessions_parser.add_argument('-c', '--column',
            help='Print a column in addition to the defaults.',
            action='append')

    sessions_parser.set_defaults(func=handle_sessions)

    # Parameters for finishing sessions
    finish_parser = subparsers.add_parser('finish', help='Finish a game session.')
    finish_parser.set_defaults(func=handle_finish)

    # Parameters for starting sessions
    starts_parser = subparsers.add_parser('start', help='Start a session.')
    starts_parser.add_argument('title', help='Title of game to start a session for.')
    starts_parser.set_defaults(func=handle_starts)

    # Parameters for searching for games
    search_parser = subparsers.add_parser('search', help='Search for a game.')
    search_parser.add_argument('title', help='Title of game to search for.')
    search_parser.add_argument('-p', '--passes', help='Show number of passes.',
            action='store_true')
    search_parser.add_argument('-e', '--eternal', help='Show whether eternal.',
            action='store_true')
    search_parser.set_defaults(func=handle_search)

    # Parameters for adding ownership of a game
    own_parser = subparsers.add_parser('own', help='Add ownership record for a game.')
    own_parser .add_argument('title', help='Title of game now owned.')
    own_parser .set_defaults(func=handle_own)

    # Parameters for selecting a game to play
    select_parser = subparsers.add_parser('select', help='Select a random game to play.')
    select_parser.add_argument('-l', '--linux', help='Only select games available on linux.',
            action='store_true')
    select_parser.add_argument('-c', '--couch',help='Only select games playable on couch.',
            action='store_true')
    select_parser.add_argument('-po', '--portable',help='Only select games playable portably.',
            action='store_true')
    select_parser.add_argument('-o', '--old', help='Only select games from before this year.',
            action='store_true')
    select_parser.add_argument('-b', '--buy', help='Include games that are not owned.',
            action='store_true')
    select_parser.add_argument('-bb', '--buy_only', help='Onlye include games that are not owned.',
            action='store_true')
    select_parser.add_argument('-sf', '--storefront', help='Only include games on specified storefront.',
            action='store', default=None)
    select_parser.add_argument('-n', help='Number of games to select.',
            action='store', default='1')
    select_parser.add_argument('-m', '--max_passes',
            help='Maximum number of times a game can be passed before it is retired.',
            action='store', default=2)
    select_parser.add_argument('-p', '--pick',
            help='Start game picking algorithm after showing selection.',
            action='store_true')
    select_parser.set_defaults(func=handle_select)

    # Parameters for importing games from Google Docs
    import_parser = subparsers.add_parser('import', help='Import games from google docs.')
    import_parser.add_argument('file_name', help='Path to file with the main list of games from Google docs.')
    import_parser.add_argument('--sessions', action='store', help='Path to file with list of sessions from Google docs.')
    import_parser.set_defaults(func=handle_import)

    # Parameters for adding games
    # Punting on this due to CSV backend
    add_parser = subparsers.add_parser('add', help='Add a new game.')
    add_parser.set_defaults(func=handle_add)

    # Parameters for printing game stats
    gamestats_parser = subparsers.add_parser('gamestats', help='Print stats on game in collection.')
    gamestats_parser.set_defaults(func=handle_gamestats)

    # Parameters for resetting game availability
    reset_parser = subparsers.add_parser('reset', help='Reset a game to be selectable.')
    reset_parser.set_defaults(func=handle_reset)
    reset_parser.add_argument('title', help='Title of game to reset.')

    # Parameters for modifying games
    # Punting on this due to CSV backend
    #add_parser = subparser.add_parser('edit', help='Edit a game.')

    args = arg_parser.parse_args()
    args.func(args)
