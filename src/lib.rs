use torrentlearn_model::Operator;
use torrentlearn_model::OperatorProvider;
use rand::Rng;


extern crate rand;
extern crate torrentlearn_model;

mod operator_compiler;
mod codegen;

#[derive(Debug)]
enum CompileError
{
	LaunchError(std::io::Error),
	ExecutionError(String,String),
	OpenError(String),
	SymbolError(String)
}

pub struct Kaleidoscope;
impl OperatorProvider for Kaleidoscope
{
	fn get_random(&self) -> Operator{unimplemented!()}
	fn get_slice(&self) -> &[Operator] {unimplemented!()}
	//dynamic dispatch as no paramitzed types in a trait
	fn random(&self,rng: &mut Rng) ->Operator {unimplemented!()}
	fn random_with_successors(&self,rng: &mut Rng, suc: u8) -> Operator { unimplemented!()}
}

impl Kaleidoscope
{
    pub fn new()->Kaleidoscope
    {
        Kaleidoscope
    }
}
