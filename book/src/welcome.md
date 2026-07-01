# Welcome

Welcome to the **"C to Rust Migration Book"**!

This course will teach you techniques for migration C codebases to Rust.\
You'll learn how to maintain a safe mixed C-Rust codebase, incrementally migrate
modules, translate common C idioms to Rust and learn the debugging tools to use
when things inevitably break.

We assume prior knowledge of Rust and some C.

You'll build up your knowledge in small, manageable steps. By the end of the
course, you will have solved many exercises, and should be prepared to migrate
even larger C codebases to Rust.

## Methodology

This course is based on the "learn by doing" principle.\
It has been designed to be interactive and hands-on.

[Mainmatter](https://mainmatter.com/rust-consulting/) developed this course to
be delivered in a classroom setting, over 4 days: each attendee advances through
the lessons at their own pace, with an experienced instructor providing
guidance, answering questions and diving deeper into the topics as needed.\
If you'd like to organize a private session for your company, please
[get in touch](https://mainmatter.com/contact/).

You can also take the course on your own, but we recommend you find a friend or
a mentor to help you along the way should you get stuck.

<!--You can find solutions for all exercises in the
[`solutions` branch of the GitHub repository](https://github.com/mainmatter/migrating-c-to-rust/tree/solutions).-->

## Formats

You can go through the course material [in the browser]([TODO: insert URL]) or
[download it as a PDF file]([TODO: insert URL]), for offline reading.

## Structure

On the left side of the screen, you can see that the course is divided into
sections. To verify your understanding, each section is paired with an exercise
that you need to solve.

You can find the exercises in the
[companion GitHub repository](https://github.com/mainmatter/migrating-c-to-rust).\
Before starting the course, make sure to clone the repository to your local
machine:

```bash
git clone https://github.com/mainmatter/migrating-c-to-rust
```

We also recommend you work on a branch, so you can easily track your progress
and pull in updates from the main repository, if needed:

```bash
cd c-to-rust-migration-book
git checkout -b my-solutions
```

All exercises are located in the `exercises` folder. Each exercise is structured
as a Rust package. The package contains the exercise itself, instructions on
what to do (in `src/lib.rs`), and a test suite to automatically verify your
solution.

### Tools

To work through this course, you'll need:

- [**Rust**](https://www.rust-lang.org/tools/install). If `rustup` is already
  installed on your system, run `rustup update` (or another appropriate command
  depending on how you installed Rust on your system) to ensure you're running
  on the latest stable version.
- _(Optional but recommended)_ An IDE with Rust autocompletion support. We
  recommend one of the following:
  - [RustRover](https://www.jetbrains.com/rust/);
  - [Visual Studio Code](https://code.visualstudio.com) with the
    [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
    extension.
- A C compiler. The one provided by your Operating System will be good enough.

### Workshop runner

To verify your solutions, we've also provided a tool to guide you through the
course: the `wr` CLI, short for "workshop runner". Install `wr` by following the
instructions on
[its website](https://mainmatter.github.io/rust-workshop-runner/).

Once you have `wr` installed, open a new terminal and navigate to the top-level
folder of the repository. Run the `wr` command to start the course:

```bash
wr
```

`wr` will verify the solution to the current exercise.\
Don't move on to the next section until you've solved the exercise for the
current one.

> We recommend committing your solutions to Git as you progress through the
> course, so you can easily track your progress and "restart" from a known point
> if needed.

Enjoy the course!

## Author

This course was written by [Jonas Kruckenberg](https://jonaskruckenberg.de),
Engineering Consultant at
[Mainmatter](https://mainmatter.com/rust-consulting/).\
Jonas Kruckenberg is a systems engineer and technologist focused on
next-generation computing infrastructure. He is the lead author of k23, an
experimental high-reliability operating system. As a TC39 Invited Expert, he
helps shape the future of web by bringing non-browser perspectives to JavaScript
language standardization.

[Mainmatter](https://mainmatter.com/rust-consulting/) is an engineering consultancy specializing in Rust. We offer engineering services, training, and mentorship to companies that aim to migrate from C to Rust or introduce Rust into an existing codebase.
