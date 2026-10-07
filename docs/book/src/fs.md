# fs

files and directories, path pieces, JSON and CSV; relative paths start at the current directory

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### tables

```zil
# ls(dir?), glob(pat) give name, type, size, modified
# glob: * and ? within a name, ** across directories
```

## Functions

| function | description |
|---|---|
| [`read_file(path: str)`](#read_file) | file contents as a string |
| [`write_file(path: str, v: any)`](#write_file) | replace a file with v as text; a list is one item per line, a table is CSV |
| [`append_file(path: str, v: any)`](#append_file) | add v to the end of a file as text, creating it if needed; a list is one item per line |
| [`exists(path: str)`](#exists) | whether a file or directory exists |
| [`is_dir(path: str)`](#is_dir) | whether path is a directory |
| [`ls(dir?: str)`](#ls) | a directory's entries as a table: name, type, size, modified |
| [`glob(pat: str)`](#glob) | paths matching pat as a table like ls; * and ? match within a name, ** across directories |
| [`file_size(path: str)`](#file_size) | size in bytes, as a quantity |
| [`mtime(path: str)`](#mtime) | when the file was last modified |
| [`rm(path: str)`](#rm) | delete a file or an empty directory |
| [`mv(from: str, to: str)`](#mv) | move or rename a file or directory |
| [`cp(from: str, to: str)`](#cp) | copy a file |
| [`mkdir(path: str)`](#mkdir) | create a directory and any missing parents |
| [`path_join(a: str, b: str, ...)`](#path_join) | join path pieces with the OS separator |
| [`basename(path: str)`](#basename) | the last piece of a path, or nil |
| [`dirname(path: str)`](#dirname) | everything before the last piece, or nil |
| [`ext(path: str)`](#ext) | the extension without its dot, or nil |
| [`stem(path: str)`](#stem) | the last piece without its extension, or nil |
| [`abspath(path: str)`](#abspath) | path made absolute from the current directory; need not exist |
| [`cwd()`](#cwd) | the current directory |
| [`cd(dir?: str)`](#cd) | change the current directory, home by default; returns the new one |
| [`from_json(s: str)`](#from_json) | parse JSON: objects become maps, arrays lists |
| [`to_json(v: any)`](#to_json) | v as JSON; values JSON lacks (quantities, fractions, dates) become strings |
| [`from_csv(s: str, header?: bool)`](#from_csv) | parse CSV; with a header row (the default) a table, else a list of rows. Numbers become numbers, empty cells nil |
| [`to_csv(rows: table\|list)`](#to_csv) | CSV text from a table, a list of maps, or a list of lists |

### read_file

`read_file(path: str)`: file contents as a string

```zil
read_file("notes.txt").lines.len
read_file("data.json").from_json
```

See also: [write_file](fs.md#write_file), [lines](text.md#lines), [from_csv](fs.md#from_csv)

### write_file

`write_file(path: str, v: any)`: replace a file with v as text; a list is one item per line, a table is CSV

```zil
write_file("out.txt", [1, 2, 3])
write_file("files.csv", ls())
```

See also: [read_file](fs.md#read_file), [append_file](fs.md#append_file)

### append_file

`append_file(path: str, v: any)`: add v to the end of a file as text, creating it if needed; a list is one item per line

```zil
append_file("log.txt", ["started {now}"])
```

See also: [write_file](fs.md#write_file)

### exists

`exists(path: str)`: whether a file or directory exists

```zil
exists("Cargo.toml")
```

See also: [is_dir](fs.md#is_dir)

### is_dir

`is_dir(path: str)`: whether path is a directory

```zil
is_dir("src")
```

See also: [exists](fs.md#exists)

### ls

`ls(dir?: str)`: a directory's entries as a table: name, type, size, modified

```zil
ls()
ls("src").where(|f| f.type == "file").name
```

See also: [glob](fs.md#glob), [sort_by](data/tables.md#sort_by)

### glob

`glob(pat: str)`: paths matching pat as a table like ls; * and ? match within a name, ** across directories

```zil
glob("**/*.rs").size.sum
glob("*.csv").name
```

See also: [ls](fs.md#ls)

### file_size

`file_size(path: str)`: size in bytes, as a quantity

```zil
file_size("Cargo.lock") to KiB
```

See also: [mtime](fs.md#mtime), [human_bytes](math/formatting.md#human_bytes)

### mtime

`mtime(path: str)`: when the file was last modified

```zil
now - mtime("Cargo.lock") to h
```

See also: [file_size](fs.md#file_size)

### rm

`rm(path: str)`: delete a file or an empty directory

```zil
rm("out.txt")
```

See also: [mv](fs.md#mv), [mkdir](fs.md#mkdir)

### mv

`mv(from: str, to: str)`: move or rename a file or directory

```zil
mv("draft.txt", "final.txt")
```

See also: [cp](fs.md#cp), [rm](fs.md#rm)

### cp

`cp(from: str, to: str)`: copy a file

```zil
cp("data.csv", "backup/data.csv")
```

See also: [mv](fs.md#mv)

### mkdir

`mkdir(path: str)`: create a directory and any missing parents

```zil
mkdir("out/2026/10")
```

See also: [rm](fs.md#rm)

### path_join

`path_join(a: str, b: str, ...)`: join path pieces with the OS separator

```zil
path_join("a", "b.txt")
# → "a\\b.txt"
```

See also: [dirname](fs.md#dirname), [basename](fs.md#basename)

### basename

`basename(path: str)`: the last piece of a path, or nil

```zil
"a/b/c.txt".basename
# → "c.txt"
```

See also: [dirname](fs.md#dirname), [stem](fs.md#stem)

### dirname

`dirname(path: str)`: everything before the last piece, or nil

```zil
"a/b/c.txt".dirname
# → "a/b"
```

See also: [basename](fs.md#basename)

### ext

`ext(path: str)`: the extension without its dot, or nil

```zil
"a/b.tar.gz".ext
# → "gz"
"README".ext
# → nil
```

See also: [stem](fs.md#stem)

### stem

`stem(path: str)`: the last piece without its extension, or nil

```zil
"a/b.tar.gz".stem
# → "b.tar"
```

See also: [ext](fs.md#ext), [basename](fs.md#basename)

### abspath

`abspath(path: str)`: path made absolute from the current directory; need not exist

```zil
abspath("src")
```

See also: [cwd](fs.md#cwd)

### cwd

`cwd()`: the current directory

```zil
cwd()
```

See also: [abspath](fs.md#abspath), [cd](fs.md#cd)

### cd

`cd(dir?: str)`: change the current directory, home by default; returns the new one

```zil
cd("src")
cd("..")
cd()
```

See also: [cwd](fs.md#cwd)

### from_json

`from_json(s: str)`: parse JSON: objects become maps, arrays lists

```zil
"\{\"a\": [1, 2.5, null]}".from_json
# → {a: [1, 2.5, nil]}
```

See also: [to_json](fs.md#to_json), [parse](core.md#parse)

### to_json

`to_json(v: any)`: v as JSON; values JSON lacks (quantities, fractions, dates) become strings

```zil
{a: [1, nil], b: 5 km}.to_json
# → "{\"a\":[1,null],\"b\":\"5 km\"}"
```

See also: [from_json](fs.md#from_json)

### from_csv

`from_csv(s: str, header?: bool)`: parse CSV; with a header row (the default) a table, else a list of rows. Numbers become numbers, empty cells nil

```zil
"name,n\nx,1".from_csv
# → table([{name: "x", n: 1}])
"1,2\n3,4".from_csv(false)
# → [[1, 2], [3, 4]]
```

See also: [to_csv](fs.md#to_csv), [table](data/tables.md#table)

### to_csv

`to_csv(rows: table|list)`: CSV text from a table, a list of maps, or a list of lists

```zil
[{a: 1, b: "x,y"}].to_csv
# → "a,b\n1,\"x,y\"\n"
[[1, 2], [3, 4]].to_csv
# → "1,2\n3,4\n"
```

See also: [from_csv](fs.md#from_csv)

## More examples

### fs

```zil
# JSON to a value
"\{\"a\": [1, 2]}".from_json.a.sum
# → 3
# CSV to a table
"name,n\nx,1\ny,2".from_csv.n.sum
# → 3
# path pieces
["a/b.tar.gz".stem, "a/b.tar.gz".ext]
# → ["b.tar", "gz"]
```
