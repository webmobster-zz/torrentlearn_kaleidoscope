use super::parse::ParseTree;
use super::parse::ParseTree::{SomeStatement,ListStatement};
use super::parse::Statement;
use super::parse::Statement::{SingleStatement,VecStatement,ReduceStatement,MapStatement};
use super::parse::Data;
use super::parse::Data::{Val,Pos};
use super::parse::Position;
use super::parse::Position::{ConstPos,VarPos};


pub mod llvminterface;

const position_one:u8=0;
const position_two:u8=1;
const position_three:u8=2;
const position_four:u8=3;


trait Codegen {
    fn codegen(&self, context: &mut LLVMContext, module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue;
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
    context: *mut u8,
    kaleidoscope_jit: *mut u8,
    ir_builder: *mut u8
}

/*impl Drop for LLVMContext{
    fn drop(&mut self)
    {
        panic!("unimplemented")
    }
}*/

pub struct LLVMModule{
    module: *mut u8,
    function_pass_analyzer: *mut u8
}

/*impl Drop for LLVMModule{
    fn drop(&mut self)
    {
        panic!("unimplemented")
    }
}*/

pub struct FunctionContext
{
    array: Option<LLVMValue>
}

impl Codegen for ParseTree {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue {
        match self {
            &ListStatement(ref statements) => panic!("wut"),
            &SomeStatement(ref statement) => {
                let (function,args) = llvminterface::generate_function_proto(context,module);
                let statement = statement.codegen(context,module,arg);
                return llvminterface::finalize_function(context,module,statement);
            }
        }
    }
}

impl Codegen for Statement {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue {
        match self {
            &SingleStatement(ref operator,ref pos,ref data) =>{
                llvminterface::generate_single_statement(*operator,pos.codegen(context,module,args),data.codegen(context,module,args))
            }
            _ => panic!("unimplemented")
        }
    }
}

impl Codegen for Data {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue {
        match self {
            &Val(val) => llvminterface::generate_constant_val(context,val,args),
            &Pos(ref position) => position.codegen(context,module,args)
        }
    }
}
impl Codegen for Position {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut FunctionContext) -> LLVMValue {
        match self {
            &ConstPos(val) => llvminterface::generate_constant_val(context,val,args),
            &VarPos(val) => llvminterface::load_array_cell(val,context,module,args)
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
