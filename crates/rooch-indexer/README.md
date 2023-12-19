Rooch indexer provides `Full Data` & `Real-time` indexer services, including `Transaction`, `Event` and `State` data directly generated from chain, and stores data based on SQLite.

SQLite uses a single file to store it's data and only requires minimal tools to be installed.

## Architecture

//TODO updated
![enhanced_FN](https://user-images.githubusercontent.com/1904567/277620523-224ece33-183b-4d9f-bb75-822afd08eac0.png)

## Steps to run locally

### Running standalone indexer

1. DB setup, under `rooch/crates/rooch-indexer` run:

```sh
# an example DATABASE_URL is "~/.rooch/local/roochdb/indexer.sqlite"
diesel setup --database-url="<DATABASE_URL>"
diesel migration generate {table_name}
diesel migration run --database-url="<DATABASE_URL>"
```

Note that you'll need an existing database for the above to work. Replace `table` with the name of the database created.


### Testcase

todo
