# Decklist
Decklist is a simple TUI program to compare your Magic: the Gathering collection with a decklist and see which cards you are missing.  The missing cards can be exported to a text file or copied directly to your clipboard for bulk entry into must online card markets.

Decklist is currently available for Linux and Windows.  It could supposedly be built for Mac as well, but I don't have one to test on.  Feel free to try building it from source.

Decklist is still very much in the "it works on my computer" phase.  It's a pretty simple program, but you may still encounter bugs.  Report them by opening an issue or shooting me an email.

## Usage
At minimum, load a collection file and a decklist file using the file browsers in the respective tabs.  A list of missing cards will be generated on the **Missing** tab.  The missing list can be exported to a file named *[decklist-name]_missing.txt* by pressing **F**, or copied directly to your clipboard with **C**.

### Collection
At the moment, Decklist only supports collections in the Moxfield export CSV format because that's the only file I have available.  Open an issue if you would like a different format supported - having an example will make it relatively easy to add.

### Decklist
Decklist supports the standard plain text format of:
```
## Card Name
## Card Name
```
Missing card exports will be in the same format.

### Database
Decklist references the Scryfall bulk data list of cards to check for any mispellings among your missing cards.  Decklist uses the lightest complete database, currently just under 150 MB in size.  This feature can be disabled using the config file (see below).  The database is stored in your local data folder.  On Linux: `~/.local/share/decklist`, and on Windows: `C:\Users\[USER]\AppData\Local\decklist`.

### Configuration
Decklist features can be configured using the **config.toml** file in the user's config directory.  On Linux that should be `~/.config/decklist`.  On Windows that will be `C:\Users\[USER]\AppData\Roaming\decklist`.
**use_database** - Set to false to prevent Decklist from downloading or loading a database file from Scryfall.  All features related to that database will be disabled.
**database_path** - Directory where the database file is stored (and checked to auto-load on startup).
**database_age_limit** - Maximum age of database file (in days) before Decklist downloads a new one.  The process is pretty quick, but it doesn't need to be that frequent, new cards are only added every so often.
**database_num** - The number of database files to keep.  Decklist keeps 3 around by default in case the latest file has breaking changes to the API.  You can manually load an older file from the **Database** tab.  Decklist will automatically delete old files beyond this number.
**collection_path** This is the path to your collection file that Decklist will attempt to load automatically on startup.  This can be updated from within the program when successfully loading a collection file in the **Collection** tab.

## Installation
Decklist can be installed by downloading the binaries from the *Releases* page.  If you have the Rust toolchain installed, you can build from source by cloning this repository and running `cargo build --release`.
