# AOC_DRIVER

This crate provides helpful functions and macros for completing Advent of Code

The main functionality is provided by the `aoc_complete` macro, which pretty much does everything for you...

```rust
fn main() {
	aoc_driver::aoc_complete! {
		session_file: ".session.txt"
		input_dir: "input"
		challenges: [
			{
				"2019-1-1": year2019::day1::part1,
				tests: [
					{ name: "1", input: "12", output: "2" }
					{ name: "2", input: "14", output: "2" }
				]
			}
			{
				"2019-1-2": year2019::day1::part2,
				tests: [
					{ name: "1", input: "100756", output: "50346" }
				]
			}
		]
	}
}
```

This will:
 - Run 2 tests for the 2019 day 1 part 1 challenge with the provided input
 - Download the official input data using your session token
 - Submit an answer using the input data
 - Tell you if your answer was correct
 - If you were correct it continues and runs the next provided tests and challenge etc.