# <div align=center>kasm</div>
<div align=center>
kasm (is) Another Submission Multitool (for Moodle). Pronounced <i>chasm</i>. Made for terminal warriors.
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

You then have the choice between two workflows.

**1. CSV & ZIP Workflow** 
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

**2. Autofetch Workflow**
- Run `kasm setup-fetch` once to
    1. define a course ID (look at the URL in your browser)
    2. save your Moodle Token to your OS's keyring (You can find the token under Moodle > Settings > Tokens > Moodle Mobile Mobile Service)
- Fetch an assignment using `kasm fetch`
    - `fetch` will display a list of assignments to select from.
- Grade using `kasm grade`
- Publish your grades automatically using `kasm push`
- Repack your feedback zip using `kasm repack` (in the master directory)
  - **Note**: `kasm repack` will **NOT** produce a .csv with Autofetch. You'll need to use `kasm push` to publish grades.

### Command line
`kasm` currently has 5 subcommands

|Subcommand | Arguments |
|-|-|
| help        | Get help for `kasm` or for any other subcommand |
| init        | Creates a master config file in the current directory |
| unpack      | Extracts a moodle zip file, filters a moodle csv for the given group and initializes a slave config in the new directory |
| repack      | Repacks the zip for upload to Moodle and constructs a grading worksheet using the slave config |
| grade       | Assigns a grade to a group. It is also able to infer the group number if you're currently in its directory. |
| setup-fetch | Saves the course ID to the master config and a Moodle API token to the user's keyring. |
| fetch       | Fetches an assignment's submissions (no parameters/interactive) |
| push        | Publishes grades (can **only** be used with fetch - not with unpack!!!) |


### Examples

**Example 1**: Plain usage with CSV and ZIP
```bash
# cd to your master directory
cd master

# init kasm (just once for multiple sheets!)
# let's say we have group 01
kasm init -g 01

# unpack sheet 01
kasm unpack -s 01 -z /path/to/moodle.zip -c /path/to/moodle.csv

# look at group 01's files
cd 'unpack_01/Übungsgruppe 01 -- Abgabeteam 01'
...

# grade group 01 with 16,384 (while inside its folder)
kasm grade 16,384

# grade group 02 with 8,192 (while inside any folder inside the unpack_XX folder)
kasm grade 8,192 02

# go back to master folder and repack sheet 01
cd ../../
kasm repack 01
```

**Example 2**: Autofetch workflow
```bash
# cd to your master directory
cd master

# setup the autofetch workflow (needs course ID and moodle token)
kasm fetch-setup

# fetch some assignment (interactive)
kasm fetch

# look at group 01's files
cd 'unpack_01/Übungsgruppe 01 -- Abgabeteam 01'
...

# grade group 01 with 16,384 (while inside its folder)
kasm grade 16,384

# publish grades AUTOMAGICALLY
kasm push

# go back to master and repack (ONLY the ZIP file will get
# repacked if you use autofetch)
cd ../../
kasm repack 01
```



## Plans

### Immediate Future
All of these would be better off as script-hooks
- [ ] ~~Recursively extract zips~~ [CANCELED]
- [ ] ~~Prepend all extracted PDFs with e.g. a grading table~~ [CANCELED]
- [ ] ~~Generate said grading table dynamically (LaTeX/Handlebars)~~ [CANCELED]

### Soon™
- [ ] Add script hooks
- [x] ~~Support unpacking groups and then repacking groups~~
- [ ] Support unpacking individuals and then repacking individuals
- [ ] Support unpacking individuals and then groupping them

### Would be cool at some point I guess
- [ ] Hardcode less stuff. Things like target directory names should be handled e.g. by Handlebars
    to make everything more easily modifiable for special cases (and if Moodle breaks *again*)
- [x] ~~Automatically download submissions (Moodle API + Token + Page ID)~~ (Only G2G for now)
- [x] ~~Automatically publish grades~~ (Only G2G for now)
- [ ] Automatically upload feedback

## Limitations
- Expects the moodle csv header to be *in German*. To change, edit
  `src/gradingtable.rs` and recompile.
- No support for nested directiories inside teams' folders when repacking (probably not a problem).
- As discussed [above](#soon), only group -> individual/group mapping is currently implemented.

## License
Licensed under EUPL-1.2-or-later. See [license](license).
