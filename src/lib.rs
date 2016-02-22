use torrentlearn_model::Operator;
use torrentlearn_model::OperatorProvider;
use torrentlearn_model::parse::ParseTree;
use rand::distributions::Weighted;
use rand::Rng;

use operator_compiler::JitCompiler;
use torrentlearn_model::parse::GeneratedResult;
use torrentlearn_model::parse::AllOperators;

extern crate rand;
extern crate torrentlearn_model;

mod operator_compiler;
mod codegen;


pub struct Kaleidoscope {
    compiler: JitCompiler,
    enabled_operators: Vec<Weighted<AllOperators>>,
    combination_cost_calculator: fn(u64,u64)-> u64,
    base_cost_calculator: fn(&AllOperators)-> u64

}

impl OperatorProvider for Kaleidoscope
{
	fn random(&mut self,mut rng: &mut Rng) ->Operator {
        let generated = torrentlearn_model::parse::generate_function(&mut self.enabled_operators,self.base_cost_calculator,&mut rng);
        match generated {
            GeneratedResult::Tree(parsetree) => self.compiler.compile_operator(parsetree,self.base_cost_calculator,self.combination_cost_calculator),
            GeneratedResult::SpecialOperator(op) => op,
        }
    }
	fn random_with_successors(&mut self,mut rng: &mut Rng, suc: u8) -> Operator {
        let generated = torrentlearn_model::parse::generate_function_with_sucessors(&mut self.enabled_operators,self.base_cost_calculator,&mut rng,suc);
        match generated {
            GeneratedResult::Tree(parsetree) => self.compiler.compile_operator(parsetree,self.base_cost_calculator,self.combination_cost_calculator),
            GeneratedResult::SpecialOperator(op) => op,
        }
    }
    fn combine(&mut self, mut parts: Vec<ParseTree>) -> Operator {
        let mut final_tree = parts.pop().unwrap();
        for tree in parts {
            final_tree.append(tree);
        }
        self.compiler.compile_operator(final_tree,self.base_cost_calculator,self.combination_cost_calculator)
    }
    fn split(&mut self, mut parts: ParseTree, point: usize) -> (Operator,Operator){
        let second = parts.split_off(point).unwrap();
        (self.compiler.compile_operator(parts,self.base_cost_calculator,self.combination_cost_calculator) ,self.compiler.compile_operator(second,self.base_cost_calculator,self.combination_cost_calculator))
    }

}

impl Kaleidoscope
{
    pub fn new(module_rotate_period: u16, enabled_operators: Vec<Weighted<AllOperators>>, combination_cost_calculator: fn(u64,u64)-> u64, base_cost_calculator: fn(&AllOperators) -> u64)->Kaleidoscope
    {
        Kaleidoscope{compiler: JitCompiler::new(module_rotate_period),enabled_operators:enabled_operators,combination_cost_calculator:combination_cost_calculator,base_cost_calculator:base_cost_calculator}
    }
}
