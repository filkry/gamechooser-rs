import sqlite3, sys, datetime, csv
from datetime import date

use_levenshtein = False
try:
    import Levenshtein
    use_levenshtein = True
except:
    pass

def create_schema(conn):
    with conn:
        conn.execute('''CREATE TABLE game
            (id integer primary key autoincrement,
            title text, release_year integer, linux integer,
            play_more integer, couch integer, portable integer, passes integer,
            via text, eternal integer, next_valid_date datetime)''')

        conn.execute('''CREATE TABLE own
            (game_id integer, storefront text,
            foreign key (game_id) references game(id))''') 

        conn.execute('''CREATE TABLE sessions
            (game_id integer, started datetime,
            outcome text,
            foreign key (game_id) references game(id))''')

def dict_from_row(row):
    return dict(zip(row.keys(), row))

def dicts_from_rows(rows):
    return list(map(dict_from_row, rows))

def import_gdoc_sessions(conn, rows):
    with conn:
        skipped = 0
        for title, start_date, status, keep, notes in rows:
            if title == 'Title' or len(title) == 0:
                continue
            gid = None 
            for game_id in conn.execute("""SELECT id FROM game WHERE title = ?""", (title,)):
                gid = game_id[0]
                break

            if gid is None:
                skipped += 1
                gid = add_game(conn, title, None, None, None, None,
                        0, None, False, [])

            # munge
            status = None if status == '' else status
            if len(start_date) == 0:
                start_date = None
            else:
                day, month, year = start_date.split('/')
                start_date = datetime.datetime(int(year), int(month), int(day))

            conn.execute("""INSERT INTO
                sessions(game_id, started, outcome)
                VALUES (?, ?, ?)""", (gid, start_date, status))

    print("Added %i games due to mismatched titles." % skipped)

def search_score(candidate, title):
    cl = candidate.lower()
    tl = title.lower()
    if cl in tl:
        return 0
    elif use_levenshtein:
        return Levenshtein.distance(cl, tl)
    else:
        return sys.maxsize


def search_game(conn, title):
    with conn:
        games = list(conn.execute('''SELECT * FROM game'''))
        
        games = sorted(games, key = lambda x: search_score(title, x[1]))

        return dicts_from_rows(games)

def add_ownership(conn, game_id, storefront):
    conn.execute('''insert into own(game_id, storefront) values(?, ?)''',
            (game_id, storefront))

def add_game(conn, title, release_year, linux, play_more, couch, portable, passes, via, eternal, storefronts):
    c = conn.cursor()

    c.execute('''insert into
            game(title, release_year, linux, play_more,
                couch, portable, passes, via, eternal)
            values(?, ?, ?, ?, ?, ?, ?, ?, ?)''',
            (title, release_year, linux, play_more,
                couch, portable, passes, via, eternal))
    game_id = c.lastrowid
    assert(game_id != None)

    for sf in storefronts:
        if sf == '':
            continue
        c.execute('''insert into
            own(game_id, storefront) values(?, ?)''',
            (game_id, sf))

    conn.commit()
    return game_id

def import_gdoc_games(conn, rows):
    for title, release_year, linux, play_more, owned_on, couch, portable, passes, via in rows:
        if title == 'title' or title == '':
            continue

        # munge data
        release_year = None if release_year == '' else int(release_year)
        linux = linux == 1 or linux == '1'
        play_more = play_more == 1 or play_more == '1'
        couch = couch == 1 or couch == '1'
        portable = portable == 1 or portable == '1'
        passes = 0 if passes == '' or passes == 'eternal' else int(passes)
        eternal = 1 if passes == 'eternal' else 0

        add_game(conn, title, release_year, linux, play_more, couch, portable, passes, via, eternal, owned_on.split(','))


def dump_csvs(conn, fn_prefix):
    with conn:
        with open("%s_game.csv" % fn_prefix, 'w') as game:
            writer = csv.writer(game, lineterminator='\n')
            writer.writerow(['id', 'title', 'release_year', 'linux',
                'play_more', 'couch', 'portable', 'passes', 'via',
                'eternal', 'next_valid_date'])
            for row in conn.execute('SELECT * FROM game'):
                writer.writerow([row['id'], row['title'], row['release_year'],
                        row['linux'], row['play_more'], row['couch'], row['portable'],
                        row['passes'], row['via'], row['eternal'],
                        row['next_valid_date']])

        with open("%s_own.csv" % fn_prefix, 'w') as own:
            writer = csv.writer(own, lineterminator='\n')
            writer.writerow(['game_id', 'storefront'])
            for row in conn.execute('SELECT * FROM own'):
                writer.writerow([row['game_id'], row['storefront']])

        with open("%s_session.csv" % fn_prefix, 'w') as session:
            writer = csv.writer(session, lineterminator='\n')
            writer.writerow(['game_id', 'started', 'outcome'])
            for row in conn.execute('SELECT * FROM sessions'):
                writer.writerow([row['game_id'], row['started'], row['outcome']])

def load_csvs(conn, fn_prefix):
    with conn:
        with open("%s_game.csv" % fn_prefix, 'r') as game:
            reader = csv.reader(game)
            for i, row in enumerate(reader):
                if i == 0: # skip title line
                    continue
                # handle addition of rows
                if len(row) == 9:
                    row.append(None)
                row = [(None if r == '' else r) for r in row]
                conn.execute('''insert into game(id, title, release_year,
                    linux, play_more, couch, portable, passes, via, eternal,
                    next_valid_date)
                    values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)''', row)
                    
        with open("%s_own.csv" % fn_prefix, 'r') as own:
            reader = csv.reader(own)
            for i, row in enumerate(reader):
                if i == 0: # skip title line
                    continue
                conn.execute('''insert into own(game_id, storefront)
                    values (?, ?)''', row)

        with open("%s_session.csv" % fn_prefix, 'r') as session:
            reader = csv.reader(session)
            for i, row in enumerate(reader):
                if i == 0: # skip title line
                    continue
                conn.execute('''insert into sessions(game_id, started,
                    outcome)
                    values (?, ?, ?)''', row)

def storefronts(conn, gid):
    with conn:
        rows = conn.execute('SELECT storefront FROM own WHERE game_id=?', (gid,))
        return [row['storefront'] for row in rows]

def select_random_games(conn, n = 1, before_this_year = None, linux = None,
        play_more = True, couch = None, portable = None, max_passes = 2, owned=True,
        exclude_ids = [], storefront = None):
    with conn:
        # Construct query
        conditions = []

        if before_this_year is True:
            conditions.append('release_year < ' + str(date.today().year))
        elif before_this_year is False:
            conditions.append('release_year == ' + str(date.today().year))
        else:
            conditions.append('release_year <= ' + str(date.today().year))

        if linux == True:
            conditions.append('linux == 1')
        elif linux == False:
            conditions.append('linux == 0')

        if play_more == True:
            conditions.append('play_more == 1')
        elif play_more == False:
            conditions.append('play_more == 0')

        if couch == True:
            conditions.append('couch == 1')
        elif couch == False:
            conditions.append('couch == 0')

        if portable == True:
            conditions.append('portable == 1')
        elif portable == False:
            conditions.append('portable == 0')

        if owned == True:
            conditions.append('storefront NOT NULL')
        elif owned == False:
            conditions.append('storefront IS NULL')

        if storefront is not None:
            conditions.append('storefront == "%s"' % storefront)

        conditions.append('(passes <= ' + str(max_passes) + ' OR eternal == 1)')

        select = '''SELECT id, title, release_year,
            linux, play_more, couch, portable, passes, via, eternal,
            next_valid_date
            FROM game LEFT OUTER JOIN own ON own.game_id=game.id'''

        query = select + ' WHERE ' + ' AND '.join(conditions) 

        day, month, year = date.today().day, date.today().month, date.today().year
        # exclude games that have been delayed for another proposal
        query += ''' AND (next_valid_date IS NULL OR
            date(next_valid_date) <= date("%04i-%02i-%02i"))''' % (year, month, day)

        # get active session game_ids
        active_ids = [row['game_id'] for row in show_sessions(conn)]

        # exclude games with ids in exclude list
        # and games with active sessions
        # this is slower than doing this with joins, but for our
        # purposes it is sufficient
        if len(exclude_ids + active_ids) > 0:
            query += ' AND id NOT IN (' + ','.join([str(i) for i in exclude_ids + active_ids]) + ')'

        query += ' ORDER BY RANDOM()'

        if n != 0:
            query += ' LIMIT ' + str(n)

        return dicts_from_rows(conn.execute(query))

def show_sessions(conn, active = True, status = None,
        session_year = date.today().year):
    with conn:
        conditions = []

        if active is True:
            conditions.append('outcome is null OR outcome == ""')
        elif active is False:
            conditions.append('outcome is not null AND outcome != ""')

        if status is not None:
            conditions.append('outcome == "' + status + '"')

        if session_year is not None:
            conditions.append('((started BETWEEN "%i-01-01" AND "%i-12-31") OR outcome is null OR outcome == "")' % (session_year, session_year))

        query = '''SELECT * FROM 
            sessions AS s JOIN game AS g ON s.game_id=g.id WHERE ''' + ' AND '.join(conditions)
        query += ' ORDER BY started'

        return dicts_from_rows(conn.execute(query))

def create_session(conn, game_id):
    with conn:
        conn.execute('INSERT INTO sessions(game_id, started) VALUES (?, ?)',
                (game_id, date.today()))


def reset_selectability(conn, game_id):
    with conn:
        conn.execute('UPDATE game SET passes = 0 WHERE id = ?', (game_id,))
        conn.execute('UPDATE game SET next_valid_date = ? WHERE id = ?', (date.today(), game_id,))

def inc_pass(conn, game_id):
    with conn:
        conn.execute('UPDATE game SET passes = passes + 1 WHERE id = ?', (game_id,))
        for row in conn.execute('SELECT passes FROM game WHERE id = ?', (game_id,)):
            return row['passes']
        return None

def set_next_valid_date(conn, game_id, next_valid_date):
    c = conn.cursor()
    c.execute('UPDATE game SET next_valid_date = ? WHERE id == ?',
            (next_valid_date, game_id))
    conn.commit()

def set_eternal(conn, game_id, val):
    with conn:
        conn.execute('UPDATE game SET eternal = ? WHERE id == ?', (val, game_id,))

def finish_session(conn, game_id, status):
    with conn:
        conn.execute('UPDATE sessions SET outcome = ? WHERE game_id == ? AND outcome == ""',
                (status, game_id))
        conn.execute('UPDATE game SET passes = 0 WHERE id == ?', (game_id,))

def retire_game(conn, game_id):
    with conn:
        conn.execute('UPDATE game SET play_more = 0 WHERE id == ?', (game_id,))
