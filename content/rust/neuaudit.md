---
tags: [rust, dev]
title: Parsing Northeastern's Graduation Audit
link: https://github.com/dustinknopoff/audit-parser
date: 2020-05-19 08:31
---

![Example Degree Audit](https://res.cloudinary.com/dknopoff/image/upload/v1589926569/Example%20Degree%20Audit.png)

> See the full example [here](https://github.com/sandboxnu/graduatenu/blob/master/scrapers/test/mock_audits/cs_audit.html)

Northeastern's graduation audit software is unattractive, unintuitive, and annoying. I've had to use it on and off for over 4 years and each time I have to use it, I desperately wish that it'll be the last time.

With that introduction, it's no wonder that I've taken it upon myself to write a parser to extract the meaningful bits in the hopes of creating a much more palatable version to use.

## Prior Art

I'd be remiss not to mention [SandboxNEU](https://www.sandboxnu.com/) which has which I shamelessly used as a reference for parsing this degree audit. They're doing a brilliant job converting the degree audit in to a full course planner with a visual drag-and-drop interface for filling all of your requirements.

As someone with 2 semesters left in my time at Northeastern, this is pretty pointless for my uses. What I'm attempting here is to have a generic JSON content extracted from the parser with the hopes of building a cleaner, more intuitive UI on top.

## What's being used.

As an unapologetic rustacean, the natural choice for language to build this parser in was Rust. I've chosen to use [pest](https://pest.rs) as a parser library. There's something incredibly appealing about a pest file containing, for the most part, all of the logic for parsing a document.

Here's the complete `.pest` file:

```
// CONSTANTS
WHITESPACE = _{ " " | NEWLINE | "\t" }
DATE = { ASCII_DIGIT{2} ~ "/" ~ ASCII_DIGIT{2} ~ "/" ~ ASCII_DIGIT{2} }

// Graduation Date
GRAD_STRING = _{"GRADUATION DATE:"}
GRADUATION_DATE = { GRAD_STRING ~ DATE }
SKIP_TO_GRAD = _{ (!"GRADUATION" ~ ANY)* }
GRAD_PARSER = { SKIP_TO_GRAD ~ GRADUATION_DATE }

// Catalog Year
CATALOG_STRING = _{"CATALOG YEAR:"}
CATALOG_NUM = { ASCII_DIGIT{6} }
CATALOG_YEAR = { CATALOG_STRING ~ CATALOG_NUM }
SKIP_TO_CATALOG = _{ (!"CATALOG" ~ ANY)* }
CATALOG_PARSER = { SKIP_TO_CATALOG ~ CATALOG_YEAR }

// Major
MAJOR_STRING = _{" - Major"}
MAJOR = @{(!MAJOR_STRING ~ ANY)*} // Requires splitting on newline

// NuPath
NAME_VAL = { ASCII_ALPHA | " " | "/"}
NUPATH_NAME = @{ (!" (" ~ NAME_VAL)* }
NUPATH_ID = {ASCII_ALPHA_UPPER{2}}
STATUS = { "OK" | "IP" | "NO" }
NUPATH_PARSER = ${ STATUS ~ WHITESPACE{0,5} ~ NUPATH_NAME ~ " (" ~ NUPATH_ID ~ ")" }

// Course List
COURSE_LIST = _{ "Course List: "}
ID = @{ (ASCII_ALPHA_UPPER | " "){4}}
COURSE_NUMBER = @{ ASCII_DIGIT{4}}
COURSE = {ID? ~ COURSE_NUMBER }
SKIP_PARENS = _{ "(" ~ ANY{11} ~ ")" }
TO = { "TO" }
COURSE_LIST_PARSER = { COURSE_LIST ~ (COURSE ~ (SKIP_PARENS | TO)?)* }

// Courses
SEASON = { "FL" | "SP" | "S1" | "S2" | "SM" }
ABBREV_YEAR = { ASCII_DIGIT{2} }
YEAR = { SEASON ~ ABBREV_YEAR }
CREDITS = { ASCII_DIGIT ~ "." ~ ASCII_DIGIT{2} }
MAYBE_IP = { ANY{8} }
COURSE_NAME = { (!WHITESPACE{2} ~ ANY)*}
COURSE_PARSER = ${ " "* ~ YEAR ~ " " ~ COURSE ~ " "{0,5} ~ CREDITS ~ MAYBE_IP ~ COURSE_NAME }

// Requirements and Status information
FLOAT = { ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT* }
EARNED_HOURS = { WHITESPACE* ~ "(" ~ FLOAT ~ "EARNED HOURS" ~ ")" }
NUM_COURSE = { ASCII_DIGIT+ }
COURSES_TAKEN = { NUM_COURSE ~ "COURSES TAKEN" }
ATTEMPTED_HOURS = { FLOAT ~ "ATTEMPTED HOURS" }
POINTS = { FLOAT ~ "POINTS" }
GPA = { FLOAT ~ "GPA" }
INFO = { EARNED_HOURS ~ COURSES_TAKEN ~ ATTEMPTED_HOURS ~ POINTS ~ GPA }

// Course, Course List, NuPath combined
COURSE_OPTION = { NUPATH_PARSER | COURSE_LIST_PARSER | COURSE_PARSER | INFO }
SKIP_TO_OPTIONS = _{ (!COURSE_OPTION ~ ANY)* }

main = { GRAD_PARSER ~ CATALOG_PARSER ~ MAJOR ~ (SKIP_TO_OPTIONS ~ COURSE_OPTION)*}
```

Kinda nice right?

## How We Get There

If only that file was created via immaculate conception. Instead it was a lot of trial and error and looking at the [GraduateNU](https://github.com/sandboxnu/graduatenu) codebase.

In fact that was the first thing I did. There's a file [types.ts](https://github.com/sandboxnu/graduatenu/blob/master/frontend/src/models/types.ts) containing all of the typescript types used in their codebase. I copied it in to a constants.rs file and just converted them in to rust equivalents (`class` => `struct`, `number` => `isize`, `string[]` => `Vec<String>`, etc.).

![Types in Rust and Typescript](https://res.cloudinary.com/dknopoff/image/upload/v1589928472/portfolio/audit-types.png)

Continuing to look through the GraduateNU codebase, there's a file [html_parser.ts](https://github.com/sandboxnu/graduatenu/blob/master/scrapers/src/html_parser.ts) which is exactly what we intend to do:

> HTML Degree Audit parser built for Northeastern University's degree audit.
> By Jacob Chvatal for Northeastern Sandbox.
>
> Filters through a Northeastern degree audit, extracting its relevant information
> to an easy to work with JSON file tracking major, graduation date,
> classes and NUPaths taken and in progress as well as requirements to take.

Jackpot!

The relevant, high level part here is:

```javascript
    for (let i: number = 0; i < lines.length; i++) {
      if (contains(lines[i], "Course List")) {
        this.add_courses_to_take(lines, i, "");
      } else if (contains(lines[i], "((FL|SP|S1|S2|SM)\\d\\d)")) {
        this.add_course_taken(lines[i]);
      } else if (contains(lines[i], "(>OK |>IP |>NO )")) {
        this.get_nupaths(lines[i]);
      } else if (contains(lines[i], "CATALOG YEAR")) {
        this.add_year(lines[i]);
      } else if (contains(lines[i], "GRADUATION DATE:")) {
        this.add_grad_date(lines[i]);
      } else if (contains(lines[i], "Major")) {
        this.add_major(lines[i]);
      }
    }
  }
```

Which leverages line counting, regex, and more. But I want the beautiful clean, pest file! So how does one go from being able to access arbitrary lines to a pest grammar, which amongst other things, never backtracks?

## Converting to Pest

As a rule, start from the beginning. Looking at the sample audit and the code block above, the first significant piece of data is the `GRADUATION DATE`.

First we'll define what whitespace means:

```
WHITESPACE = _{ " " | NEWLINE | "\t" }
```

Pest 'rules' are always provided inside `{}`. The `_` in front tells Pest to ignore anything that matches the pattern that follows. The `|` represents 'OR'.

This can be read as:

> WHITESPACE is a silent rule matching " " or newlines or "\t"

Next, you'll notice that the graduation date is in the format 08/20/22, let's create a rule for that as well.

```
DATE = { ASCII_DIGIT{2} ~ "/" ~ ASCII_DIGIT{2} ~ "/" ~ ASCII_DIGIT{2} }
```

This introduces us to some more pest syntax. `ASCII_DIGIT` is a constant representing [0-9]. `{2}` following the ASCII_DIGIT says to match twice. The `~` symbol tells pest to match the following token next.

This can be read as:

> DATE is a rule matching 2 digits, then a "/", then 2 digits, then a "/", then 2 digits

Soon, we're going to be composing these rules. An important part of pest rules is that everything with the `{}` "counts". So, when we want to get just the date from this block: `GRADUATION DATE: 08/20/22` we couldn't have a rule:

```
GRADUATION_DATE = { "GRADUATION_DATE" ~ DATE }
```

As pest would see the string "GRADUATION_DATE" as a something worth tracking.

For this reason, we'll write it like this

```
GRAD_STRING = _{"GRADUATION DATE:"}
GRADUATION_DATE = { GRAD_STRING ~ DATE }
```

This can be read as:

> GRAD_STRING is a rule silently matching "GRADUATION DATE:"
>
> GRADUATION_DATE is a rule matching the rule `GRAD_STRING` and then matching the rule `DATE`

But wait! There's a whole bunch (24 lines) of text that comes before the tiny bit to care about. How do we get rid of that?

```
SKIP_TO_GRAD = _{ (!"GRADUATION" ~ ANY)* }
```

This introduces us to a couple new pest syntaxes. We'll move outside in. The `*` is similar to the glob `*` in that it means match 0 or more times. It's preceded by a statement wrapped in `()` which means for the `*` to match on the entirety of the inner statement. The `!` means "NOT" and `ANY` is a pest provided constant meaning anything.

This can be read as:

> SKIP_TO_GRAD is a rule silently matching zero or more tokens which are not "GRADUATION"

This all comes together to create:

```
// CONSTANTS
WHITESPACE = _{ " " | NEWLINE | "\t" }
DATE = { ASCII_DIGIT{2} ~ "/" ~ ASCII_DIGIT{2} ~ "/" ~ ASCII_DIGIT{2} }

// Graduation Date
GRAD_STRING = _{"GRADUATION DATE:"}
GRADUATION_DATE = { GRAD_STRING ~ DATE }
SKIP_TO_GRAD = _{ (!"GRADUATION" ~ ANY)* }
GRAD_PARSER = { SKIP_TO_GRAD ~ GRADUATION_DATE }
```

And using [pest.rs](https://pest.rs) to it test out

![Grad Parser on pest.rs](https://res.cloudinary.com/dknopoff/image/upload/v1589934035/portfolio/grad-parser.png)

This process was repeated until all of the sections GraduateNU extracted, were extracted in the pest rules.

## The Rust

```rust
#[derive(Parser)]
#[grammar = "audit.pest"]
pub struct AuditParser;
```

This is a procedural macro provided by pest which magically converts the pest file in to a rust structure.

Brilliantly, it's designed to be accessed in a recursive match statement.

```rust
 pub fn parse_audit(file: &'_ str) -> Result<AuditToJson<'_>, PestError<Rule>> {
        let main = Self::parse(Rule::main, file)?.next().unwrap();
        let mut out = AuditToJson::new();
        fn parse_inner<'a>(mut out: &mut AuditToJson<'a>, rule: Pair<'a, Rule>) {
            match rule.as_rule() {
                Rule::GRAD_PARSER => {
                    let date = rule
                        .into_inner()
                        .next() // Move in to GRAD_PARSER Steps
                        .unwrap()
                        .into_inner()
                        .next() // Skip GRAD_STRING
                        .unwrap();
                    parse_inner(out, date);
                }
                Rule::CATALOG_PARSER => {
                    // Reach in to parser and get CATALOG_NUM
                    let year = rule.into_inner().next().unwrap();
                    parse_inner(out, year);
                }
                Rule::DATE => {
                    let date = NaiveDate::parse_from_str(rule.as_str(), "%D").unwrap();
                    out.grad_date = date;
                }
// ...
```

Similar to the screenshot from [pest.rs](https://pest.rs), in the `Rule::GRAD_PARSER` match branch, we reach in to a rule `GRAD_PARSER > GRAD_STRING > GRAD_DATE`

And the result is sweet, sweet JSON!

```json
{
  "majors": [
    "Computer Science"
  ],
  "minors": [],
  "audit_year": 2020,
  "grad_date": "2022-05-20",
  "complete_nupaths": [
    "ND",
    "EI",
    "IC",
    "FQ",
    "SI",
    "AD",
  ],
  ...
```
