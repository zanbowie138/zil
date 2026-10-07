# data.tables

rows under named columns: filter, pick columns, sort; list fns work on the rows

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### types

```zil
# table
[{a: 1, b: 2}] to table
# → table([{a: 1, b: 2}])
```

### access

```zil
# t.col (a column)
[{a: 1}, {a: 2}].table.a
# → [1, 2]
# t[i] (a row)
[{a: 1}, {a: 2}].table[1]
# → {a: 2}
```

## Functions

| function | description |
|---|---|
| [`table(rows: list\|table)`](#table) | a table from a list of maps; columns are every key, missing cells nil |
| [`where(t: table, f: fn)`](#where) | keep rows where f(row) is truthy |
| [`select(t: table, col: str, ...)`](#select) | keep only these columns, in this order |
| [`reject(t: table, col: str, ...)`](#reject) | drop these columns |
| [`sort_by(t: table, col: str)`](#sort_by) | rows sorted by a column |
| [`sort_by_desc(t: table, col: str)`](#sort_by_desc) | rows sorted by a column, largest first |

### table

`table(rows: list|table)`: a table from a list of maps; columns are every key, missing cells nil

```zil
[{a: 1}, {a: 2, b: 3}].table
# → table([{a: 1, b: nil}, {a: 2, b: 3}])
[{a: 1}] to table
# → table([{a: 1}])
```

See also: [select](../data/tables.md#select), [list](../core.md#list)

### where

`where(t: table, f: fn)`: keep rows where f(row) is truthy

```zil
[{n: 1}, {n: 5}].table.where(|r| r.n > 2)
# → table([{n: 5}])
```

See also: [filter](../data/lists.md#filter), [sort_by](../data/tables.md#sort_by)

### select

`select(t: table, col: str, ...)`: keep only these columns, in this order

```zil
[{a: 1, b: 2, c: 3}].table.select("c", "a")
# → table([{c: 3, a: 1}])
```

See also: [reject](../data/tables.md#reject)

### reject

`reject(t: table, col: str, ...)`: drop these columns

```zil
[{a: 1, b: 2, c: 3}].table.reject("b")
# → table([{a: 1, c: 3}])
```

See also: [select](../data/tables.md#select)

### sort_by

`sort_by(t: table, col: str)`: rows sorted by a column

```zil
[{n: 3}, {n: 1}].table.sort_by("n")
# → table([{n: 1}, {n: 3}])
```

See also: [sort_by_desc](../data/tables.md#sort_by_desc), [sort](../data.md#sort)

### sort_by_desc

`sort_by_desc(t: table, col: str)`: rows sorted by a column, largest first

```zil
[{n: 1}, {n: 3}].table.sort_by_desc("n")
# → table([{n: 3}, {n: 1}])
```

See also: [sort_by](../data/tables.md#sort_by)

## More examples

### tables

```zil
# a column as a list
[{n: 1}, {n: 5}].table.n
# → [1, 5]
# list fns see rows as maps
[{n: 1}, {n: 5}].table.map(|r| r.n * 2)
# → [2, 10]
# sort, pick, count
[{f: "a", kb: 3}, {f: "b", kb: 9}].table.sort_by_desc("kb").f
# → ["b", "a"]
```
