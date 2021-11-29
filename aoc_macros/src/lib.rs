mod config;

use quote::quote;
use syn::{
	parse::Parse, parse_macro_input, token::Comma, Expr, ExprLit, ExprPath, Lit, LitStr, Path,
};

use crate::config::gen_from_config;

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

#[derive(Debug)]
struct ConfigInput {
	config_path: LitStr,
	session_path: LitStr,
}

impl Parse for ConfigInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let config_path = Parse::parse(input)?;
		let _ = Comma::parse(input)?;
		let session_path = Parse::parse(input)?;

		Ok(Self {
			config_path,
			session_path,
		})
	}
}

/// Usage:
/// ```rust
/// run!(year, day, part, function, input_path);
/// ```
/// eg.
/// ```rust
/// run!(2021, 1, 1, year2020::day1::part1, "input/2020/1.txt")
/// ```
/// The `function` may also be a string literal
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

/// Usage:
/// ```rust
/// run!(year, day, part, function, input_path);
/// ```
/// eg.
/// ```rust
/// run!(2021, 1, 1, year2020::day1::part1, "input/2020/1.bin")
/// ```
/// The `function` may also be a string literal.
/// The `_bytes` version calls the function with a &[u8]
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

/// Usage:
/// ```rust
/// run!(year, day, part, function, test_name, test_input, expected);
/// ```
/// eg.
/// ```rust
/// run!(2021, 1, 1, year2020::day1::part1, "simple", "123", "456");
/// ```
/// The `function` may also be a string literal.
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

/// Usage:
/// ```rust
/// run!(year, day, part, function, test_name, test_input, expected);
/// ```
/// eg.
/// ```rust
/// run!(2021, 1, 1, year2020::day1::part1, "simple", "123", "456");
/// ```
/// The `function` may also be a string literal.
/// The `_bytes` version calls the function with a &[u8]
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

/// Usage:
/// ```rust
/// execute_config!(config_file, session_file);
/// ```
/// eg.
/// ```rust
/// execute_config!("aoc_config.toml", ".session_token.txt");
/// ```
/// Runs tests, downloads inputs, and submits answers based off a given `TOML` file
///
/// The session file must contain only the session token taken from your browser
///
/// Challenge headers take the form of: `[challenges.{year}-{day}-{part}]`
///
/// Test headers take the form of: `[challenges.{year}-{day}-{part}.tests.{name}]`
///
/// Example config:
/// ```toml
/// input_dir = "input"
///
/// [challenges.2019-1-1]
/// function = "year2019::day1::part1"
///
/// [challenges.2019-1-1.tests.1]
/// input = "12"
/// output = "2"
///
/// [challenges.2019-1-1.tests.2]
/// input = "14"
/// output = "2"
///
/// [challenges.2019-1-2]
/// function = "year2019::day1::part2"
///
/// [challenges.2019-1-2.tests.0]
/// input = "100756"
/// output = "50346"
/// ```
#[proc_macro]
pub fn execute_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as ConfigInput);
	let output = gen_from_config(input.config_path.value(), input.session_path.value());
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
