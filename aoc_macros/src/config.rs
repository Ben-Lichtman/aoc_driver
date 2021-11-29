use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs::{create_dir_all, read_to_string},
	path::{Path, PathBuf},
};
use syn::parse_str;
use toml::from_str;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
	input_dir: PathBuf,
	challenges: HashMap<String, ChallengeDesc>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChallengeDesc {
	function: String,
	tests: Option<HashMap<String, Test>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Test {
	input: String,
	output: String,
}

pub fn gen_from_config(config_path: String, session_path: String) -> TokenStream {
	let session = read_to_string(session_path).unwrap();
	let session = session.trim();

	let config = read_to_string(config_path).unwrap();
	let config = from_str::<Config>(&config).unwrap();

	let inputs_dir = Path::new(&config.input_dir);

	let mut challenges = config
		.challenges
		.into_iter()
		.map(|(name, desc)| {
			let name_split = name.split('-').collect::<Vec<_>>();
			let (year, day, part) = match name_split.as_slice() {
				[y, d, p] => (
					y.parse::<u16>().expect("invalid year"),
					d.parse::<u8>().expect("invalid day"),
					p.parse::<u8>().expect("invalid part"),
				),
				_ => panic!("challenge names invalid in toml (y-d-p)"),
			};
			(year, day, part, desc)
		})
		.collect::<Vec<_>>();
	challenges.sort_by_cached_key(|(y, d, p, _)| (*y, *d, *p));

	let all = challenges
		.into_iter()
		.map(|(year, day, part, desc)| {
			let function = parse_str::<syn::Path>(&desc.function).expect("invalid function path");

			let mut input_file = PathBuf::from(inputs_dir);
			input_file.push(format!("{}", year));

			create_dir_all(&input_file).unwrap();

			input_file.push(format!("{}.txt", day));

			let tests = desc
				.tests
				.into_iter()
				.flat_map(IntoIterator::into_iter)
				.map(|(name, test)| {
					let input = test.input;
					let output = test.output;
					quote! {
						let input = #input;
						let output = #output;
						if !::aoc_driver::test!(#year, #day, #part, #function, #name, input, output) {
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
					if !::aoc_driver::get_input(&session, #year, #day, input_path) {
						println!("Could not get input file");
						return;
					}
				}

				let displayed = ::aoc_driver::run!(#year, #day, #part, #function, #input_file_token);
				if !::aoc_driver::post_answer(&session, #year, #day, #part, &displayed) {
					println!("> INCORRECT");
					return;
				}
				println!("> CORRECT");
			}
		})
		.collect::<TokenStream>();

	quote! {
		let session = #session;

		#all
	}
}
