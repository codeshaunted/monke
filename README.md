# monke
monke is a command line packing/unpacking tool for Bloons TD 6 save files. It can be used on files such as `Profile.Save`, `identity`, `player-index-PRODUCTION` and anything else with the same NinjaKiwi file encoding. In Bloons TD 6 specifically, it allows for modification of save files to give monkey money, instant monkeys, change levels, give exp, etc. You use this tool at your own risk, as abusing it could cause your NinjaKiwi account to be flagged or banned.

# Building
## Disclaimer
Building monke from source will require a moderate amount of C++ knowledge. If you're not up to building it from source, you can grab a pre-built binary from the [GitHub releases page](https://github.com/codeshaunted/monke/releases).
## Required Software
To download and build monke you will need to have Git, CMake 3.12 or greater and either one of the MSVC or GCC compilers (Clang is incompatible with my cryptography library choice, much to my dismay).
## Cloning the Repository
Clone the repository into your directory of choice using the command below. Do not miss the `--recursive` flag, it is very important and your build will fail without it!
```
git clone --recursive https://github.com/codeshaunted/monke
```
## Building the Repository
Change directories into the cloned repository and run the set of commands below.
```bash
:: Create the build directory
mkdir build
cd build

:: Run CMake to generate make files
cmake ..

:: Run CMake with build flag to build
cmake --build .
```
If everything worked properly, you should now have a build folder with a monke executable inside of it. If not, feel free to open an issue on GitHub with a detailed account of your issue and everything that led to it.

# Usage
monke is a very simple program with only two commands, `unpack` and `pack`.
## `unpack`
The `unpack` command will unpack a NinjaKiwi encoded file into a new file. It requires that you give it an input file path, an output file path, and the password to unpack the file with. An example of `unpack` usage is shown below.
```bash
:: Unpack a file in the working directory named 'Profile.Save' into a new file named 'Profile.Save.json', using the password '11'
monke unpack Profile.Save Profile.Save.json 11
```
## `pack`
The `pack` command will pack a file into a new NinjaKiwi encoded file. It requires that you give it an input file path, an output file path, and the password to pack the file with. An example of `pack` usage is shown below.
```bash
:: Pack a file in the working directory named 'Profile.Save.json' into a new file named 'Profile.Save', using the password '11'
monke pack Profile.Save.json Profile.Save 11
```

# Passwords
To pack or unpack a save file you need a password, the following is a list of passwords corresponding to NinjaKiwi games. In all cases so far, the password corresponds to NinjaKiwi's internal "app ID" for the game.
| Game          | Password    |
| ------------- | ----------- |
| Tower Keepers | `8`         |
| Bloons TD 6   | `11`        |
