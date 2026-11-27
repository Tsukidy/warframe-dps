"""Minimal Lua table-literal parser (enough for the Module:Stances/data entries; stance_e70.py).

Lua semantics kept: an unquoted name in value position is an unset global and evaluates to nil (`None`), except
`true` / `false` (07 §4.1: Blind Justice Forward Block input 3 `Procs` holds the unquoted `Knockdown`, which is no
proc); string escapes `\\n`, `\\t`, `\\ddd` (decimal byte) and `\\<char>`; non-ASCII text is kept as UTF-8.
"""
import re

ESC = re.compile(r'\\(\d{1,3}|.)', re.S)


def unescape(s):
    def one(m):
        c = m.group(1)
        if c.isdigit():
            return chr(int(c))
        return {'n': '\n', 't': '\t', 'r': '\r'}.get(c, c)
    return ESC.sub(one, s)


TOK = re.compile(r'\s*(?:(--[^\n]*)|("(?:[^"\\]|\\.)*")|([{}\[\],=])|(-?\d+(?:\.\d+)?)|([A-Za-z_][A-Za-z0-9_]*))')


def tokens(s):
    pos = 0
    out = []
    while pos < len(s):
        m = TOK.match(s, pos)
        if not m or m.end() == pos:
            if s[pos:].strip() == '':
                break
            raise ValueError(f'bad lua at {pos}: {s[pos:pos+40]!r}')
        pos = m.end()
        com, st, p, num, ident = m.groups()
        if com:
            continue
        if st is not None:
            out.append(('s', unescape(st[1:-1])))
        elif p:
            out.append(('p', p))
        elif num:
            out.append(('n', float(num) if '.' in num else int(num)))
        elif ident:
            out.append(('i', ident))
    return out


def parse(s):
    t = tokens(s)
    i = 0

    def val():
        nonlocal i
        k, v = t[i]
        if k == 'p' and v == '{':
            return table()
        i += 1
        if k == 'i':
            return {'true': True, 'false': False}.get(v)  # any other identifier is an unset global: nil
        return v

    def table():
        nonlocal i
        assert t[i] == ('p', '{')
        i += 1
        keyed = {}
        pos = []
        while t[i] != ('p', '}'):
            if t[i] == ('p', '['):
                i += 1
                key = val()
                assert t[i] == ('p', ']')
                i += 1
                assert t[i] == ('p', '=')
                i += 1
                keyed[key] = val()
            elif t[i][0] == 'i' and t[i + 1] == ('p', '='):
                key = t[i][1]
                i += 2
                keyed[key] = val()
            else:
                pos.append(val())
            if t[i] == ('p', ','):
                i += 1
        i += 1
        if keyed and pos:
            keyed['_pos'] = pos
            return keyed
        return keyed if keyed else pos
    return val()


def block(text, name):
    """The table literal assigned to `\t<name> = {` (top-level entry of the module)."""
    m = re.search(r'^\t(?:' + re.escape(name) + r'|\["' + re.escape(name) + r'"\]) = \{', text, re.M)
    start = m.end() - 1
    depth = 0
    i = start
    in_str = False
    while True:
        c = text[i]
        if in_str:
            if c == '\\':
                i += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == '-' and text[i:i + 2] == '--':
            i = text.index('\n', i)
            continue
        elif c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return parse(text[start:i + 1])
        i += 1
