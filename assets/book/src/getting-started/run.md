# Run it

```bash
cd app
cargo run -p gateway
```

That is the whole setup. No database server, no configuration file, no schema to
create by hand.

You should see:

```
INFO gateway: applied migration 20260101000001
INFO gateway: applied migration 20260101000002
INFO gateway: applied migration 20260101000003
INFO gateway: applied migration 20260101000004
INFO gateway: applied migration 20260101000005
INFO gateway: listening on http://127.0.0.1:8080
```

Five migrations, one per table. They ran because the database was empty. Stop the
process and start it again and you will not see those lines — Diesel keeps a
record of which migrations it has applied, in a table it manages itself.

## Where the database went

A file called `admin.db` in whichever directory you ran the command from. It is
SQLite, and the engine is compiled into the binary, which is why nothing had to
be installed.

Delete the file to start over. The next run recreates it.

## Check it is alive

```bash
curl -s localhost:8080/health
```

```json
{ "ok": true }
```

`/health` is one of only three routes that work without a token. Try one that
does not:

```bash
curl -s -o /dev/null -w '%{http_code}\n' localhost:8080/users
```

```
401
```

That is the system working. [A walkthrough with curl](../practice/walkthrough.md)
takes it from here.

## Run the tests

```bash
cargo test --workspace
```

```
tests: 20 passed, 0 failed
```

The tests use an in-memory database, so they neither need nor touch `admin.db`.
