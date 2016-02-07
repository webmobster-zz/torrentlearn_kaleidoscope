use torrentlearn_model::UUID;
use torrentlearn_model::GlobalState;
use torrentlearn_model::LocalState;
use torrentlearn_model::SpecialOperator;
use torrentlearn_model::Operator;

use torrentlearn_model::parse::ParseTree;
use codegen::{LLVMContext,LLVMModule,FunctionContext};
use codegen::llvminterface;
use codegen::Codegen;

use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::io::Read;
use self::core::str::FromStr;

extern crate core;

pub struct JitCompiler {
    context: LLVMContext,
    current_module: Arc<Mutex<LLVMModule>>

}

impl JitCompiler {
    fn new() -> JitCompiler {
        unimplemented!();
    }
    fn compile_operator(&mut self,operator: UnCompiledOperator,  store_for_mutation: bool) -> Operator {
        unimplemented!();
    }

    fn rotate_module(&mut self) {
        self.current_module= Arc::new(Mutex::new(LLVMModule::with_context(&mut self.context)));
    }
}

fn generate_uuid()->UUID {
    unimplemented!();
}

//Use the module to keep track of uses for removal and dropping
fn compile(context: &mut LLVMContext, current_module: &mut Arc<Mutex<LLVMModule>>,parsetree: ParseTree) -> (fn(&mut LocalState) -> bool, Arc<Mutex<LLVMModule>>) {
    let mut module = current_module.lock().unwrap();
    let (function,mut arg_temp) = llvminterface::generate_function_proto(context,&mut module);
    let mut args = FunctionContext{ array: arg_temp.remove(0) };
    let statement = parsetree.codegen(context,&mut module,&mut args);
    llvminterface::finalize_function(statement,context,&mut *module);
    unimplemented!();
    //let pointer = llvminterface::get_pointer(statement,context,&mut module);
    //return (pointer,current_module.clone())

}


pub struct UnCompiledOperator
{
    parse_tree: ParseTree
}

pub trait Compiler
{
	fn compile(&self, ordered_code : Vec<String>, uuid: UUID) -> fn(&mut[u8]) -> bool;
}
