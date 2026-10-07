# sys

script arguments, environment variables, shell commands, HTTP GET, exit codes, network access

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### scripts

```zil
# zil script.zil a b  →  args() is ["a", "b"]
# sh fails on a non-zero exit; run never does
# currency rates and translation stay offline until allow_network_access()
```

## Functions

| function | description |
|---|---|
| [`args()`](#args) | command-line arguments after the script (or after `-e code`), as strings |
| [`env() / env(name: str)`](#env) | an environment variable, or nil; with no name, all of them as a map |
| [`exit(code?: int)`](#exit) | stop the program with an exit code (default 0) |
| [`sh(cmd: str)`](#sh) | run a shell command (sh -c, or cmd /C on Windows) and return its output, trailing newline trimmed; errors on a non-zero exit |
| [`run(cmd: str)`](#run) | run a shell command and return {code, out, err}; never fails on the exit code |
| [`fetch(url: str)`](#fetch) | HTTP GET a URL and return the body as a string; errors on a non-2xx status |
| [`allow_network_access()`](#allow_network_access) | let currency rates and translation go online for the rest of the session; until then they only use cached data |

### args

`args()`: command-line arguments after the script (or after `-e code`), as strings

```zil
args()
# → []
```

See also: [env](sys.md#env)

### env

`env() / env(name: str)`: an environment variable, or nil; with no name, all of them as a map

```zil
env("ZIL_SURELY_UNSET")
# → nil
```

See also: [args](sys.md#args)

### exit

`exit(code?: int)`: stop the program with an exit code (default 0)

```zil
exit(1)
```

See also: [args](sys.md#args)

### sh

`sh(cmd: str)`: run a shell command (sh -c, or cmd /C on Windows) and return its output, trailing newline trimmed; errors on a non-zero exit

```zil
sh("git rev-parse --short HEAD")
sh("ls").lines.len
```

See also: [run](sys.md#run)

### run

`run(cmd: str)`: run a shell command and return {code, out, err}; never fails on the exit code

```zil
run("git status").code
```

See also: [sh](sys.md#sh)

### fetch

`fetch(url: str)`: HTTP GET a URL and return the body as a string; errors on a non-2xx status

```zil
fetch("https://api.github.com/repos/rust-lang/rust").from_json.stargazers_count
```

See also: [from_json](fs.md#from_json)

### allow_network_access

`allow_network_access()`: let currency rates and translation go online for the rest of the session; until then they only use cached data

```zil
allow_network_access()
```

See also: [fetch](sys.md#fetch)

## More examples

### sys

```zil
# arguments after the script
args()
# → []
# is a variable set?
env("ZIL_SURELY_UNSET") == nil
# → true
# how many env vars
env().len > 0
# → true
```
