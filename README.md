# Aoc Helpers
All functionality requires AoC session cookie, which you can get from you browser after logging in

(look in developer tools)

The most obvious way to use this library is with the `calculate_and_post` function

```rust
use aoc_driver::*;

fn solution(i: &str) -> String { unimplemented!() }

let session = "<session cookie>";
calculate_and_post(
    session,
    2020,
    1,
    Part1,
    Some("inputs/2020/1.txt"),
    solution
).unwrap();
```

There is an even faster way though using the `aoc_magic` macro

```rust
use aoc_driver::*;

fn solution(i: &str) -> String { unimplemented!() }

let session = "<session cookie>";
aoc_magic!(session, 2020:1:1, solution).unwrap()
```

This macro does the same as the above function call (including creating an `inputs` directory), but more concisely