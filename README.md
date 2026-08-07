# monke

> [!WARNING]
> Editing save data may cause your Ninja Kiwi account to be flagged or banned. Use at your own risk.

Pack and unpack Ninja Kiwi save files.

Download the latest binary from the [releases page](https://github.com/codeshaunted/monke/releases).

```sh
monke unpack Profile.Save -o Profile.Save.json --game btd6
monke pack Profile.Save.json -o Profile.Save --game btd6
monke inspect Profile.Save
```

Supported games are `battd`, `btd6`, and `tower-keepers`. For other games, use `--password` instead of `--game`. Feel free to make a [pull request](https://github.com/codeshaunted/monke/pulls) to add support for another game. Pass `--force` to replace an existing output file. Run `monke --help` for all options.
