use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::HashMap, fs::read_to_string, hash::Hash, path::PathBuf};
use syn::{
	braced, bracketed,
	parse::Parse,
	parse_macro_input,
	token::{Brace, Bracket, Colon, Comma},
	Expr, ExprLit, ExprPath, Ident, Lit, LitStr, Path,
};

#[derive(Debug)]
struct RunInput {
	year: Expr,
	day: Expr,
	part: Expr,
	function: Expr,
	input_path: Expr,
}

impl Parse for RunInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let year = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let day = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let part = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let function = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let input_path = Parse::parse(input)?;

		Ok(Self {
			year,
			day,
			part,
			function,
			input_path,
		})
	}
}

#[derive(Debug)]
struct TestInput {
	year: Expr,
	day: Expr,
	part: Expr,
	function: Expr,
	test_name: Expr,
	test_input: Expr,
	expected: Expr,
}

impl Parse for TestInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let year = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let day = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let part = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let function = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let test_name = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let test_input = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let expected = Parse::parse(input)?;

		Ok(Self {
			year,
			day,
			part,
			function,
			test_name,
			test_input,
			expected,
		})
	}
}

#[derive(Debug)]
struct TestBytesInput {
	year: Expr,
	day: Expr,
	part: Expr,
	function: Expr,
	test_name: Expr,
	test_input: Expr,
	expected: Expr,
}

impl Parse for TestBytesInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let year = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let day = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let part = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let function = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let test_name = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let test_input = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let expected = Parse::parse(input)?;

		Ok(Self {
			year,
			day,
			part,
			function,
			test_name,
			test_input,
			expected,
		})
	}
}

/// Runs a given function
///
///  Usage:
/// ```rust
/// run!(year, day, part, function, input_path);
/// ```
/// eg.
/// ```rust
/// run!(2021, 1, 1, year2020::day1::part1, "input/2020/1.txt")
/// ```
/// The `function` may also be a string literal ie `"year2020::day1::part1"`
///
/// The function should be able to take in a `&str` and output something which is displayable
/// eg.
/// ```rust
/// fn example(input: &str) -> impl std::fmt::Display {
///     ...
/// }
/// ```
///
/// `input_path` must be a path to a file containing the input into the function
#[proc_macro]
pub fn run(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as RunInput);
	let year = input.year;
	let day = input.day;
	let part = input.part;
	let function = expr_to_path(input.function);
	let input_path = input.input_path;
	let output = quote! {
		{
			println!("\n==> Running {} day {} part {}:", #year, #day, #part);
			let input_path = ::std::path::Path::new(#input_path);
			let input = ::std::fs::read_to_string(input_path).unwrap();
			let result = #function(input.as_ref());
			println!("{}", result);
			result.to_string()
		}
	};
	proc_macro::TokenStream::from(output)
}

/// Runs a given function with byte input
///
/// Usage:
/// ```rust
/// run_bytes!(year, day, part, function, input_path);
/// ```
/// eg.
/// ```rust
/// run_bytes!(2021, 1, 1, year2020::day1::part1, "input/2020/1.bin")
/// ```
/// The `function` may also be a string literal.
/// The `_bytes` version calls the function with a &[u8]
///
/// The function should be able to take in a `&[u8]` and output something which is displayable
/// eg.
/// ```rust
/// fn example(input: &[u8]) -> impl std::fmt::Display {
///     ...
/// }
/// ```
///
/// `input_path` must be a path to a file containing the input into the function
#[proc_macro]
pub fn run_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as RunInput);
	let year = input.year;
	let day = input.day;
	let part = input.part;
	let function = expr_to_path(input.function);
	let input_path = input.input_path;
	let output = quote! {
		{
			println!("\n==> Running {} day {} part {}:", #year, #day, #part);
			let input_path = ::std::path::Path::new(#input_path);
			let input = ::std::fs::read(input_path).unwrap();
			let result = #function(input.as_ref());
			println!("{}", result);
			result.to_string()
		}
	};
	proc_macro::TokenStream::from(output)
}

/// Runs a test with given input
///
/// Usage:
/// ```rust
/// test!(year, day, part, function, test_name, test_input, expected);
/// ```
/// eg.
/// ```rust
/// test!(2021, 1, 1, year2020::day1::part1, "simple", "123", "456");
/// ```
/// The `function` may also be a string literal ie `"year2020::day1::part1"`
///
/// The function should be able to take in a `&str` and output something which is displayable
/// eg.
/// ```rust
/// fn example(input: &str) -> impl std::fmt::Display {
///     ...
/// }
/// ```
#[proc_macro]
pub fn test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as TestInput);
	let year = input.year;
	let day = input.day;
	let part = input.part;
	let function = expr_to_path(input.function);
	let test_name = input.test_name;
	let test_input = input.test_input;
	let expected = input.expected;
	let output = quote! {
		{
			println!("\n==> Testing {} day {} part {} test \"{}\":", #year, #day, #part, #test_name);
			let expected = #expected;
			let result = #function(#test_input.as_ref());
			let result_string = result.to_string();
			if result_string == expected {
				println!("> Test returned:\n{}\n> SUCCESS", expected);
				true
			}
			else {
				println!("> Test returned:\n{}\n> Expected:\n{}\n> FAIL", result_string, expected);
				false
			}
		}
	};
	proc_macro::TokenStream::from(output)
}

/// Runs a test with given byte input
///
/// Usage:
/// ```rust
/// test_bytes!(year, day, part, function, test_name, test_input, expected);
/// ```
/// eg.
/// ```rust
/// test_bytes!(2021, 1, 1, year2020::day1::part1, "simple", b"123", "456");
/// ```
/// The `function` may also be a string literal ie `"year2020::day1::part1"`
/// The `_bytes` version calls the function with a &[u8]
///
/// The function should be able to take in a `&[u8]` and output something which is displayable
/// eg.
/// ```rust
/// fn example(input: &[u8]) -> impl std::fmt::Display {
///     ...
/// }
/// ```
#[proc_macro]
pub fn test_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as TestBytesInput);
	let year = input.year;
	let day = input.day;
	let part = input.part;
	let function = input.function;
	let test_name = input.test_name;
	let test_input = input.test_input;
	let expected = input.expected;
	let output = quote! {
		{
			println!("\n==> Testing {} day {} part {} test \"{}\":", #year, #day, #part, #test_name);
			let expected = #expected;
			let result = #function(#test_input.as_ref());
			let result_string = result.to_string();
			if result_string == expected {
				println!("> Test returned:\n{}\n> SUCCESS", expected);
				true
			}
			else {
				println!("> Test returned:\n{}\n> Expected:\n{}\n> FAIL", result_string, expected);
				false
			}
		}
	};
	proc_macro::TokenStream::from(output)
}

fn expr_to_path(input: Expr) -> Path {
	// panic!("{:#?}", input);
	match input {
		Expr::Path(ExprPath { path, .. }, ..) => path,
		// Expr::Path(p) => p.path,
		Expr::Lit(ExprLit {
			lit: Lit::Str(l), ..
		}) => l.parse().expect("invalid input as function"),
		_ => panic!("invalid input as function"),
	}
}

#[derive(Debug)]
struct CompleteInput {
	session_file: LitStr,
	input_dir: LitStr,
	challenges: Vec<Challenge>,
}

impl Parse for CompleteInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let map = parse_key_value(input)?;

		let session_file = map
			.get(&Key::Ident(format_ident!("session_file")))
			.and_then(|v| match v {
				Value::LitStr(l) => Some(l),
				_ => None,
			})
			.expect("no session file provided");

		let input_dir = map
			.get(&Key::Ident(format_ident!("input_dir")))
			.and_then(|v| match v {
				Value::LitStr(l) => Some(l),
				_ => None,
			})
			.expect("no input dir provided");

		let challenges = map
			.get(&Key::Ident(format_ident!("challenges")))
			.and_then(|v| match v {
				Value::Array(b) => Some(b),
				_ => None,
			})
			.into_iter()
			.flat_map(|v| v.iter())
			.map(|m| {
				let (name, path) = m
					.iter()
					.filter_map(|(k, v)| match k {
						Key::Lit(x) => Some((x, v)),
						_ => None,
					})
					.filter_map(|(k, v)| match v {
						Value::Path(x) => Some((k, x)),
						_ => None,
					})
					.next()
					.expect("No challenge name given");

				let tests = m
					.get(&Key::Ident(format_ident!("tests")))
					.and_then(|v| match v {
						Value::Array(a) => Some(a),
						_ => None,
					})
					.into_iter()
					.flat_map(|v| v.iter())
					.map(|m| {
						let name = m
							.get(&Key::Ident(format_ident!("name")))
							.and_then(|v| match v {
								Value::LitStr(l) => Some(l),
								_ => None,
							})
							.expect("test name not provided");

						let input = m
							.get(&Key::Ident(format_ident!("input")))
							.and_then(|v| match v {
								Value::LitStr(l) => Some(l),
								_ => None,
							})
							.expect("test input not provided");

						let output = m
							.get(&Key::Ident(format_ident!("output")))
							.and_then(|v| match v {
								Value::LitStr(l) => Some(l),
								_ => None,
							})
							.expect("test output not provided");

						Test {
							name: name.clone(),
							input: input.clone(),
							output: output.clone(),
						}
					})
					.collect::<Vec<_>>();

				Challenge {
					name: name.clone(),
					function: path.clone(),
					tests,
				}
			})
			.collect::<Vec<_>>();

		Ok(CompleteInput {
			session_file: session_file.clone(),
			input_dir: input_dir.clone(),
			challenges,
		})
	}
}

#[derive(Debug)]
struct Challenge {
	name: LitStr,
	function: Path,
	tests: Vec<Test>,
}

#[derive(Debug)]
struct Test {
	name: LitStr,
	input: LitStr,
	output: LitStr,
}

/// Runs all AoC tests and solutions from a given config
///
/// Example usage:
/// ```
/// fn main() {
///     aoc_driver::aoc_complete! {
///         session_file: ".session.txt"
///         input_dir: "input"
///         challenges: [
///             {
///                 "2019-1-1": year2019::day1::part1,
///                 tests: [
///                     { name: "1", input: "12", output: "2" }
///                     { name: "2", input: "14", output: "2" }
///                 ]
///             }
///             {
///                 "2019-1-2": year2019::day1::part2,
///                 tests: [
///                     { name: "1", input: "100756", output: "50346" }
///                 ]
///             }
///         ]
///     }
/// }
/// ```
/// The `session_file` must be a text file containing your session cookie string (you can find this through inspect element in your browser)
///
/// The `input_dir` is the directory where inputs will be cached while running challenges
///
/// For each challenge in `challenges` there must be only one field which is not the `tests` field - this will have the key be the name of the challenge in the form `"{year}-{day}-{part}"` and the value must be a path to the function that will be run for the challenge
///
/// The function should be able to take in a `&str` and output something which is displayable
///
/// eg.
/// ```rust
/// fn example(input: &str) -> impl std::fmt::Display {
///     ...
/// }
/// ```
#[proc_macro]
pub fn aoc_complete(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as CompleteInput);

	let session = read_to_string(input.session_file.value()).expect("could not read session file");

	let input_dir = input.input_dir.value();

	let challenges = input
		.challenges
		.into_iter()
		.map(|c| {
			let name = c.name.value();
			let name_split = name.split('-').collect::<Vec<_>>();
			let (year, day, part) = match name_split.as_slice() {
				[y, d, p] => (
					y.parse::<u16>().expect("invalid year"),
					d.parse::<u8>().expect("invalid day"),
					p.parse::<u8>().expect("invalid part"),
				),
				_ => panic!("challenge names invalid (y-d-p)"),
			};
			(year, day, part, c.function, c.tests)
		})
		.map(|(y, d, p, f, tests)| {
			let mut input_file = PathBuf::from(&input_dir);
			input_file.push(format!("{}", y));
			input_file.push(format!("{}.txt", d));

			let tests = tests
				.into_iter()
				.map(|t| {
					let name = t.name;
					let input = t.input;
					let output = t.output;
					quote! {
						let input = #input;
						let output = #output;
						if !::aoc_driver::test!(#y, #d, #p, #f, #name, input, output) {
							return;
						}
					}
				})
				.collect::<TokenStream>();

			let input_file_token = input_file.to_str().unwrap();
			quote! {
				#tests

				let input_path = ::std::path::Path::new(#input_file_token);
				if !input_path.is_file() {
					if !::aoc_driver::get_input(&session, #y, #d, input_path) {
						println!("Could not get input file");
						return;
					}
				}

				let displayed = ::aoc_driver::run!(#y, #d, #p, #f, #input_file_token);
				if !::aoc_driver::post_answer(&session, #y, #d, #p, &displayed) {
					println!("> INCORRECT");
					return;
				}
				println!("> CORRECT");
			}
		})
		.collect::<TokenStream>();

	let output = quote! {
		let session = #session;

		#challenges
	};

	proc_macro::TokenStream::from(output)
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Key {
	Ident(Ident),
	Lit(LitStr),
}

impl Parse for Key {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let lookahead = input.lookahead1();
		if lookahead.peek(Ident) {
			return Ok(Self::Ident(input.parse()?));
		}
		if lookahead.peek(LitStr) {
			return Ok(Self::Lit(input.parse()?));
		}
		Err(syn::Error::new(input.span(), "incorrect key type"))
	}
}

#[derive(Debug)]
enum Value {
	Path(Path),
	LitStr(LitStr),
	Braced(HashMap<Key, Value>),
	Array(Vec<HashMap<Key, Value>>),
}

impl Parse for Value {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		if input.fork().parse::<Path>().is_ok() {
			return Ok(Self::Path(input.parse()?));
		}
		if input.fork().parse::<LitStr>().is_ok() {
			return Ok(Self::LitStr(input.parse()?));
		}
		if input.peek(Brace) {
			let content;
			braced!(content in input);
			let items = parse_key_value(&content)?;
			return Ok(Self::Braced(items));
		}
		if input.peek(Bracket) {
			let bracket_content;
			bracketed!(bracket_content in input);
			let mut items = Vec::new();
			while bracket_content.peek(Brace) {
				let brace_content;
				braced!(brace_content in bracket_content);
				let brace_contents = parse_key_value(&brace_content)?;
				items.push(brace_contents)
			}
			return Ok(Self::Array(items));
		}
		Err(syn::Error::new(input.span(), "incorrect value type"))
	}
}

fn parse_key_value(input: syn::parse::ParseStream) -> syn::Result<HashMap<Key, Value>> {
	let mut items = HashMap::new();
	while !input.is_empty() {
		let key = input.parse::<Key>()?;
		let _ = input.parse::<Colon>()?;
		let val = input.parse::<Value>()?;
		if input.peek(Comma) {
			let _ = input.parse::<Comma>()?;
		}
		items.insert(key, val);
	}
	Ok(items)
}
