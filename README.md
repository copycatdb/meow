# meow 📟

CLI for SQL Server. psql, but with attitude.

Part of [CopyCat](https://github.com/copycatdb) 🐱

## What is this?

An interactive command-line client for SQL Server. Think `psql` or `sqlcmd`, but one that doesnt make you want to throw your laptop out the window.

```bash
$ meow -S localhost,1433 -U sa
Password: ****
Connected to SQL Server 2022 (16.0.4135.4)

mydb> SELECT TOP 5 name, salary FROM employees ORDER BY salary DESC;
┌──────────────┬──────────┐
│ name         │ salary   │
├──────────────┼──────────┤
│ Alice Chen   │ 185000   │
│ Bob Kumar    │ 172000   │
│ Carol Smith  │ 168000   │
│ Dave Johnson │ 155000   │
│ Eve Williams │ 149000   │
└──────────────┴──────────┘
5 rows (12ms)

mydb> \dt
Tables in mydb:
  employees    (4 columns, ~10000 rows)
  departments  (3 columns, ~50 rows)
  orders       (8 columns, ~1.2M rows)
```

## Why not sqlcmd?

Have you *used* sqlcmd? The one where:
- Output alignment is a suggestion, not a feature
- You have to type `GO` after every statement like its 1995
- Tab completion is a myth
- Ctrl+C doesnt cancel, it exits
- The "new" version requires .NET 6 runtime

meow is what sqlcmd would be if it was designed by someone who actually uses a terminal.

## Features (planned)

- Syntax highlighting
- Auto-complete (tables, columns, functions)
- Pretty-printed tables (with actual alignment)
- `\d` describe commands (like psql)
- Query history with search
- `.output` to file/CSV/JSON
- No `GO` required. Were not animals.

## Status

🚧 Coming soon. Currently sharpening claws.

## Attribution

Inspired by [psql](https://www.postgresql.org/docs/current/app-psql.html), the CLI that every other database CLI wishes it could be. And a gentle roast of [sqlcmd](https://learn.microsoft.com/en-us/sql/tools/sqlcmd/sqlcmd-utility), which has served us faithfully despite everything.

## License

MIT
