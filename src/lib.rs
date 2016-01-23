use torrentlearn_model::Operator;
use torrentlearn_model::OperatorProvider;
use rand::Rng;


extern crate rand;
extern crate torrentlearn_model;

mod operator_compiler;
mod parse;
mod codegen;
//TODO: Create folders if they dont exit

#[derive(Debug)]
enum CompileError
{
	LaunchError(std::io::Error),
	ExecutionError(String,String),
	OpenError(String),
	SymbolError(String)
}

struct Kaleidoscope;
impl OperatorProvider for Kaleidoscope
{
	fn get_random(&self) -> Operator{panic!("wut")}
	fn get_slice(&self) -> &[Operator] {panic!("wut")}
	//dynamic dispatch as no paramitzed types in a trait
	fn random(&self,rng: &mut Rng) ->Operator {panic!("wut")}
	fn random_with_successors(&self,rng: &mut Rng, suc: u8) -> Operator { panic!("wut")}
}
