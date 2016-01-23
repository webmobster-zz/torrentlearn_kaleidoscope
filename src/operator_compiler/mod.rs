use torrentlearn_model::UUID;
use torrentlearn_model::GlobalState;
use torrentlearn_model::LocalState;
use torrentlearn_model::SpecialOperator;
use super::parse::ParseTree;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use self::core::str::FromStr;

extern crate core;


pub struct CompiledOperator
{
	pub function: fn(&mut LocalState) -> bool,
	pub code: Option<ParseTree>,
	pub parts: Option<Vec<UUID>>,
	pub sucessors: u8,
	pub cost: u64,
	pub uuid: UUID,
	pub special: SpecialOperator

}


pub trait Compiler
{
	fn compile(&self, ordered_code : Vec<String>, uuid: UUID) -> fn(&mut[u8]) -> bool;
}
