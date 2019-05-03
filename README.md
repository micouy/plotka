# Plotka

Plotka lets you easily broadcast your data via websockets.
It can also host your JS client so that you can plot it in your browser.

It can be used as an alternative to Matplotlib, etc. You can write
your simulations/calculations/whatever in your favorite language
and pipe the results to Plotka. It will then
send it to your JS client.


## Examples

```text
$ mycalcuations | plotka --static-path . csv --headers mass velocity position
```

```text
$ echo my-results.txt | plotka --static-path . json
```


## Parsing and data formats

For now Plotka can only handle CSV and JSON.

Plotka requires each record to have to the same fields.
The records are separated with a new line.
The values are stored as either a float or an int. The fields
are strongly typed so they must have the same type
in each record. The type of the field is evaluated
based on whether the parser can parse it as an integer
or not. Numbers ending with `.0` are parsed
as floats.

### CSV
The `csv` subcommand takes 2 args.
* The `--headers` arg (obviously) allows you to specify the headers. For now
  it is required but it may change.
* The `--delimiter` arg lets you change the delimiter.

### JSON
The `json` subcommand takes no args. Each line of input has to be a valid JSON object. If, for example, the file starts
or ends with brackets, you have to trim them.


## TODO
* Make `--static-path` arg optional and set default to current dir.
* Add `--ignore-first` arg to CSV subcommand so that the user can choose whether to ignore the first line or not.
  The headers have to be set anyways.
* Add support for TSV.
* Add autocompletions and a manual.
* Release binaries.
* Add benchmarks.


## Note

Plotka is in alpha stage. Also note that I'm a hobbyist
and I created it mainly for my own needs. I'll be grateful for your feedback.
