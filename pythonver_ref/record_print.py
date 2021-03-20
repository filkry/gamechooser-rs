import math

column_aliases = {
    'release_year': 'year',
    'platforms': 'owned',
}

def alias_replace(col):
    if col in column_aliases:
        return column_aliases[col].upper()
    return col.upper()

column_format = {
    'title': '<40.40',
    'release_year': '>4',
    'linux': '>5',
    'couch': '>5',
    'portable': '>8',
    'play_more': '>4',
    'via': '<20.20',
    'platforms': '<20.20',
    'started': '>10.10',
    'outcome': '>10.10',
    'passes': '<6',
    'eternal': '<7',
}

def format_record(record, columns):
    sections = []

    for c in columns:
        if c in column_format:
            sections.append('{%s!s:%s}' % (c, column_format[c]))
        else:
            sections.append('{%s!s}' % (c,))

    format_str = '  '.join(sections)

    output = format_str.format(**record)
    return output

def prepend_num(string, num_digits, num = None):
    line = []
    if num is None:
        line.append(' ' * (num_digits + 2))
    else:
        tmp = '{0:<%i.%i}' % (num_digits+2, num_digits+2)
        line.append(tmp.format(str(num)))
    line.append(string)
    return ''.join(line)

def format_records(records, columns, header=False, nums=False):
    output = []
    num_digits = int(math.log10(len(records)))+1

    if header:
        col_record = {c: alias_replace(c) for c in columns}
        line = format_record(col_record, columns)
        if nums:
            line = prepend_num(line, num_digits)
        output.append(line)

    for i, r in enumerate(records):
        line = format_record(r, columns)
        if nums:
            line = prepend_num(line, num_digits, i+1)
        output.append(line)

    return '\n'.join(output)

