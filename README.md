# Plotka

Plotka lets you easily broadcast your data via websockets.
It can also host your JS client so that you can plot your data in a browser.

It can be used as an alternative to Matplotlib, etc. You can write
your simulations/calculations/whatever in your favorite language
and pipe the results to Plotka. It will then
send it to your JS client.


## Examples

```text
$ mysimulation | plotka --static-path . csv --headers mass velocity position
```

```text
$ echo my-results.txt | plotka --static-path . json
```


## Parsing and data formats

For now Plotka can only handle CSV and JSON. I plan to add TSV support.

Plotka requires each record to have to the same fields.
The records are separated with a new line.
The values are stored as either a float or an int. The fields
are strongly typed so they must have the same type
in each record. The type of the field is evaluated
based on whether it can be parsed as an integer
or not. Numbers ending with `.0` will be parsed
as floats.

### CSV

The `csv` subcommand takes 2 args.
* The `--headers` arg (obviously) allows you to specify the headers. For now
  it is required but it may change.
* The `--delimiter` arg lets you change the delimiter.

### JSON

The `json` subcommand takes no args. Each line of input has to be a valid JSON object. If, for example, the file starts
or ends with brackets, you have to trim them.


## Setting up a JS client

By default Plotka binds its internal server to `127.0.0.1:8080`.
You can change it by setting the `--ip-address` arg. Your static
files will be hosted at `/static/<path-to-your-file>`. Let's say your folder
looks like this:

```text
my-static-files
├── index.html
└── js
   └── main.js
```

You can access your main script via `127.0.0.1:8080/static/js/main.js`. `/` will redirect you
to `/static/index.html`, so you don't have to type it in the URL bar.

A basic JS client could look like this:

```javascript
const url = `ws://${window.location.host}/ws/`;
const ws = new WebSocket(url);

ws.onmessage = (e) => {
    const message = e.data;
    console.info("Received a message.");

    const request = JSON.parse(message);
    console.log(`Method name: ${request["method"]}`);
    console.log(`Params: ${request["params"]}`);
}
```

You can then use your plotting library of choice.

If you don't know which one to use, take a look at these:
* [ApexCharts](https://apexcharts.com/)
* [Plottable](http://plottablejs.org/)
* [AmCharts](https://www.amcharts.com/)


## Receiving update messages

Every newly connected WS client will receive a message containing the latest state of the record storage.
The messages follow the [JSON RPC](https://en.wikipedia.org/wiki/JSON-RPC) format.

* `initStorage` message contains a list of JSON objects (records) in the `data` field.

    ```text
    {
        "method": "initStorage",
        "params": {
        	"data": [
        	    { "x": 10, "y": 1.15 },
        	    { "x": 11, "y": 1.16 },
        	    { "x": 12, "y": 1.17 },
        	    ...
        	]
        }
    }
    ```
    
* `pushRecord` message contains a single JSON object in the `record` field.
    ```text
    {
    	"method": "pushRecord",
    	"params": {
   	    "record": { "x": 13, "y": 1.18 }
        }
    }
    ```


## TODO

* [ ] Fix error handling and add documentation (!).
* [ ] Make `--static-path` arg optional and set default to current dir.
* [ ] Add `--ignore-first` arg to CSV subcommand so that the user can choose whether to ignore the first line or not.
* [ ] Add support for TSV.
* [ ] Add shell autocompletion and manual.
* [ ] Release binaries.
* [ ] Add benchmarks.
* [ ] Add methods allowing for updating data in packets of constant size.

  I want to provide a mechanism allowing you to easily manage data when
  you want to plot only some constant number of data points at a time. It could be useful
  i. e. if you're using a heat map.
* [ ] Add support for regex (?).
* [ ] Add support for date parsing (?).


## Warning

Plotka will not save the data on your computer. It will be lost after you stop Plotka.
However, it may change in the future.


## Note

Plotka is in its alpha stage. Also note that I'm a hobbyist
and I created it mainly for my own needs. I'll be grateful for your feedback.
