use torrentlearn_model::parse::ParseTree;
use torrentlearn_model::parse::ParseTree::{EndSingle};
use torrentlearn_model::parse::Statement;
use torrentlearn_model::parse::Statement::{SingleStatement,VecStatement,ReduceStatement,MapStatement};
use torrentlearn_model::parse::Data;
use torrentlearn_model::parse::Data::{Val,Pos};
use torrentlearn_model::parse::Position;
use torrentlearn_model::parse::Position::{ConstPos,VarPos};
use torrentlearn_model::operator::DropHelper;
use std::sync::{Arc,Mutex};

pub mod llvminterface;

const POSITION_ONE:u8=0;
const POSITION_TWO:u8=1;
const POSITION_THREE:u8=2;
const POSITION_FOUR:u8=3;

//Use the module to keep track of uses for removal and dropping
pub fn compile(context: &mut LLVMContext, current_module: &mut Arc<Mutex<LLVMModule>>,parsetree: &mut ParseTree) -> (fn(&mut [u8]) -> bool, Arc<Mutex<LLVMModule>>) {
    let mut module = current_module.lock().unwrap();
    let (function,mut arg_temp) = llvminterface::generate_function_proto(context,&mut module);
    let mut args = FunctionContext{ array: arg_temp.remove(0) };
    let statement = parsetree.codegen(context,&mut module,&mut args);
    llvminterface::finalize_function(statement,context,&mut *module);
    unimplemented!();
    //let pointer = llvminterface::get_pointer(statement,context,&mut module);
    //return (pointer,current_module.clone())

}

pub trait Codegen {
    fn codegen(&self, context: &mut LLVMContext, module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue;
}
pub trait ArgumentCodegen {
    fn codegen(&self, context: &mut LLVMContext, module: &mut LLVMModule, args: &mut FunctionContext,argument_position:u8) -> LLVMValue;
}

//FIXME: Does this need a drop implementation (how to clean up this value)
pub struct LLVMValue(*mut u8);

/*impl Drop for LLVMValue{
    fn drop(&mut self)
    {
        panic!("unimplemented")
    }
}*/

//Don't give send as compiling on multiple threads not supported
pub struct LLVMContext{
    pub context: *mut u8,
    pub kaleidoscope_jit: *mut u8,
    pub ir_builder: *mut u8
}

impl Drop for LLVMContext{
    fn drop(&mut self)
    {
        panic!("unimplemented")
    }
}

pub struct LLVMModule{
    pub module: *mut u8,
    pub function_pass_analyzer: *mut u8
}
impl LLVMModule {
    pub fn with_context(context: &mut LLVMContext) -> LLVMModule
    {
        llvminterface::initialize_llvm_module(context)
    }
}

impl Drop for LLVMModule{
    fn drop(&mut self)
    {
        panic!("unimplemented")
    }
}
unsafe impl Send for LLVMModule{}
impl DropHelper for LLVMModule{}

pub struct FunctionContext
{
    pub array: LLVMValue
}


impl Codegen for ParseTree {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue {
        match self {
            &EndSingle(ref statement) => {
                statement.codegen(context,module,args)

            },
            _ => unimplemented!(),
        }
    }
}

impl Codegen for Statement {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue {
        match self {
            &SingleStatement(ref operator,ref pos,ref data) =>{
                llvminterface::generate_single_statement(*operator,pos.codegen(context,module,args,POSITION_ONE),data.codegen(context,module,args,POSITION_TWO))
            }
            &VecStatement(_,_,_,_) => unimplemented!(),
            &MapStatement(_,_,_,_) => unimplemented!(),
            &ReduceStatement(_,_,_,_) => unimplemented!(),

        }
    }
}

impl ArgumentCodegen for Data {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext, argument_position: u8) -> LLVMValue {
        match self {
            &Val(val) => llvminterface::generate_constant_val(context,val),
            &Pos(ref position) => position.codegen(context,module,args, argument_position)
        }
    }
}
impl ArgumentCodegen for Position {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext, argument_position: u8) -> LLVMValue {
        match self {
            &ConstPos(val) => llvminterface::generate_constant_val(context,val),
            &VarPos => llvminterface::load_array_cell(context,module,&mut args.array,argument_position)
        }
    }
}

#[cfg(test)]
mod test{
        use super::llvminterface;
        use super::LLVMContext;
        use super::LLVMModule;
        use super::Codegen;
        use super::super::parse::Data;

        fn startLLVM() -> (LLVMContext,LLVMModule)
        {
                let mut context = llvminterface::initializeLLVM();
                let module = llvminterface::initializeLLVMModule(&mut context);
                (context,module)
        }
        #[test]
        fn test_value() {
            let (mut context,mut module) = startLLVM();
            Data::Val(54).codegen(&mut context,&mut module);
        }
        /*
        #[test]
        fn test_data() {
            unimplemented!()
        }
        */
}
