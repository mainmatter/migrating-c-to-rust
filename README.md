# Migrate C to Rust, one exercise at a time

You've got a C codebase you'd like to move to Rust, but you're not sure where to
start?\
This course is for you!

You'll learn techniques for migrating C codebases to Rust by solving exercises.\
You'll learn how to maintain a safe mixed C-Rust codebase, incrementally migrate
modules, translate common C idioms to Rust, and use the debugging tools you'll
need when things inevitably break — one exercise at a time.

> [!NOTE]
> This course has been written by
> [Mainmatter](https://mainmatter.com/rust-consulting/).\
> It's one of the trainings in
> [our portfolio of Rust workshops](https://mainmatter.com/services/workshops/rust/).\
> Check out our [landing page](https://mainmatter.com/rust-consulting/) if
> you're looking for Rust consulting or training!

## Getting started

Clone this repository to your local machine:

```bash
git clone https://github.com/mainmatter/migrating-c-to-rust
```

We recommend you work on a branch, so you can easily track your progress and
pull in updates from the main repository, if needed:

```bash
cd migrating-c-to-rust
git checkout -b my-solutions
```

All exercises are located in the `exercises` folder. Each exercise is structured
as a Rust package. The package contains the exercise itself, instructions on
what to do (in `src/lib.rs`), and a test suite to automatically verify your
solution.

To verify your solutions, we've provided a tool to guide you through the course:
the `wr` CLI, short for "workshop runner". Install `wr` by following the
instructions on
[its website](https://mainmatter.github.io/rust-workshop-runner/).

Once you have `wr` installed, open a new terminal, navigate to the top-level
folder of the repository, and run:

```bash
wr
```

`wr` will verify the solution to the current exercise. Don't move on to the next
section until you've solved the exercise for the current one.

## Requirements

- **Rust** (follow instructions
  [here](https://www.rust-lang.org/tools/install)).\
  If `rustup` is already installed on your system, run `rustup update` (or
  another appropriate command depending on how you installed Rust on your
  system) to make sure you're running on the latest stable version.
- A **C compiler**. The one provided by your operating system will be good
  enough.
- _(Optional but recommended)_ An IDE with Rust autocompletion support. We
  recommend one of the following:
  - [RustRover](https://www.jetbrains.com/rust/);
  - [Visual Studio Code](https://code.visualstudio.com) with the
    [`rust-analyzer`](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer)
    extension.

<!--## Solutions

You can find the solutions to the exercises in the
[`solutions` branch](https://github.com/mainmatter/migrating-c-to-rust/tree/solutions)
of this repository.-->

## License

Copyright © 2024- Mainmatter GmbH (https://mainmatter.com), released under the
[Creative Commons Attribution-NonCommercial 4.0 International license](https://creativecommons.org/licenses/by-nc/4.0/).
</content>
</invoke>
