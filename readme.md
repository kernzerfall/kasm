# <div align=center>kasm</div>
<div align=center>
kasm (is) Another Submission Multitool (for Moodle). Pronounced <i>chasm</i>.
</div>

<br>
<div align=center>
<img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white" />
</div>

## Installation
Just grab a release for your OS and put it in a path covered by `$PATH` (or equiv.)
e.g. via

```bash
sudo install -Dm 0755 kasm /usr/local/bin/kasm
```

Alternatively, grab the source and build/install it with cargo:
```bash
cargo build -r
cargo install --locked --path .
```

## Usage

### Brief overview
- The initialization procedure creates a reusable master config, which contains info such as
  - Your exercise group id
  - The regex to match groups against
  - The structure of the zip (are we expecting groupped folders in it?)
  - A filter (regex) to only repack certain files.
- In the master directory, you create a slave by `kasm unpack`. It contains
  - A slave config (`grades.toml`)
  - The submission folders
  - The filtered grading worksheet csv
- To assign grades, use `kasm grade` or edit the `grades.toml` file.
  - `kasm grade` will infer the team automagically if you're inside
    its subfolder
  - **IMPORTANT!** Always use the format Moodle expects (especially the decimal separator).
    `kasm` doesn't parse your inputs. They are taken at face-value as strings.
- `kasm repack` (in the master directory) packs the feedback into a zip file
    (`feedback_$SHEET_$TIMESTAMP.zip`) and grades into `grades_$SHEET_$TIMESTAMP.csv`.
        Just upload these at the appropriate place in Moodle.

### What makes `kasm` different than the slew of other submission scripts?

1. `kasm` is made for the terminal warrior. Other scripts have you
    editing .csv files - which warrants a trip to `vim` (or
    godforbid a GUI editor) - and renaming directories
    to assign grades, which can mess up other open terminals inside
    them.
    - `kasm grade` takes care of all of that for you, so that you'll
      never have to leave your terminal window (except to annotate the actual
      PDFs I guess). It will work seamlessly whether you're in the `unpack`
      directory or inside a team's submission folder.
2. `kasm` works (probably™). Other scripts are currently broken on the newest
    version of moodle.
3. `kasm` is faster and more efficient.
    - It only selectively unzips files (instead of unzipping everything and
        `rm -rf`'ing what we don't need).
    - It repacks files live - without creating an intermediary `tmp` directory,
      no matter whether it filters files, adds extra directories, etc.
    - It's compiled to native code, what else do I need to say?
4. `kasm` is more convenient.
    - Due to the usage of master/slave config files, you need to type less stuff to get the same thing done.
    - If you set a filter in the master config, you don't even need to clean up junk files from the
      directories before repacking them!
5. `kasm` has [plans for the future](#plans).

### Command line
`kasm` currently has 5 subcommands

|Subcommand | Arguments |
|-|-|
| help   | Get help for `kasm` or for any other subcommand |
| init   | Creates a master config file in the current directory |
| unpack | Extracts a moodle zip file, filters a moodle csv for the given group and initializes a slave config in the new directory |
| repack | Repacks the zip for upload to Moodle and constructs a grading worksheet using the slave config |
| grade  | Assigns a grade to a group. It is also able to infer the group number if you're currently in its directory. |

## Plans

### Immediate Future
- [ ] Recursively extract zips
- [ ] Prepend all extracted PDFs with e.g. a grading table
- [ ] Generate said grading table dynamically (LaTeX/Handlebars)

### Soon™
- [ ] Add script hooks
- [ ] Support unpacking groups and then repacking groups
- [ ] Support unpacking individuals and then repacking individuals
- [ ] Support unpacking individuals and then groupping them

(Currently, the only supported configuration is unpacking groups and then
repacking individuals)

### Would be cool at some point I guess
- [ ] Hardcode less stuff. Things like target directory names should be handled e.g. by Handlebars
    to make everything more easily modifiable for special cases (and if Moodle breaks *again*)
- [ ] Automatically download submissions (Moodle API + Token + Page ID)
- [ ] Automatically upload feedback

## Limitations
- Expects the moodle csv header to be *in German*. To change, edit
  `src/gradingtable.rs` and recompile.
- No support for nested directiories inside teams' folders when repacking (probably not a problem).
- As discussed [above](#soon), only group -> individual mapping is currently implemented.

## License
Licensed under EUPL-1.2-or-later. See [license](license).
