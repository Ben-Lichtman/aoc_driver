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
