use torrentlearn_model::parse::ParseTree;
use torrentlearn_model::parse::Statement;
use torrentlearn_model::parse::ConditionalStatement;
use torrentlearn_model::parse::Statement::{SingleStatement};
use torrentlearn_model::parse::Data;
use torrentlearn_model::parse::Data::{Val,Pos};
use torrentlearn_model::parse::Position;
use torrentlearn_model::parse::Position::{ContPos,EndPos};
use torrentlearn_model::operator::DropHelper;

use std::ffi::NulError;

#[derive(Debug)]
pub enum CompileError {
    NulError(NulError)
}
impl From<NulError> for CompileError {
    fn from(err: NulError) -> CompileError {
        CompileError::NulError(err)
    }
}

pub mod llvminterface;

//Use the module to keep track of uses for removal and dropping
pub fn compile(context: &mut LLVMContext, module: &mut LLVMModule,parsetree: &mut ParseTree, name: &str) -> Result<fn(&mut [u8]) -> bool,CompileError> {
    let (function,mut arg_temp) = llvminterface::generate_function_proto(context,module,name);
    let mut args = arg_temp.remove(0);
    let statement = parsetree.codegen(context,module,&mut args);
    llvminterface::finalize_function(statement,function,module);

    //FIXME: This assumes one function per module
    llvminterface::add_module_to_jit(context,module);

    //FIXME: Text
    let pointer = try!(llvminterface::get_pointer("changeme"));
    return Ok(pointer)

}

pub trait Codegen {
    fn codegen(&self, context: &mut LLVMContext, module: &mut LLVMModule, args: &mut LLVMValue) -> LLVMValue;
}

//FIXME: Does this need a drop implementation (how to clean up this value)
//Especially with continution statements
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

//FIXME: Seperate pass manager out of module
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
impl DropHelper for Box<LLVMModule>{}

impl Codegen for ParseTree {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &ParseTree::EndSingle(ref statement) => {
                statement.codegen(context,module,args)

            },
            &ParseTree::EndConditional(ref statement) =>  {
                statement.codegen(context,module,args)
            },
            &ParseTree::Continuation(ref continuation, ref statement) =>{
                //discard previous result
                statement.codegen(context,module,args);
                continuation.codegen(context,module,args)
            }
        }
    }
}

impl Codegen for Statement {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &SingleStatement(ref operator,ref pos,ref data) =>{
                llvminterface::generate_single_statement(*operator,pos.codegen(context,module,args),data.codegen(context,module,args))
            }
        }
    }
}
impl Codegen for ConditionalStatement {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut LLVMValue) -> LLVMValue {
        let ConditionalStatement(ref operator,ref pos,ref data) = *self;
        llvminterface::generate_conditional_statement(*operator,pos.codegen(context,module,args),data.codegen(context,module,args))

    }
}

impl Codegen for Data {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &Val(val) => llvminterface::generate_constant_val(context,val),
            &Pos(ref position) => position.codegen(context,module,args)
        }
    }
}
impl Codegen for Position {
    fn codegen(&self, context: &mut LLVMContext,module: &mut LLVMModule, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &ContPos(ref next) => {
                let array_index = next.codegen(context,module,args);
                llvminterface::generate_cont_pos(context, args,array_index)
            },
            &EndPos(val) => llvminterface::generate_end_pos(context,args,val)
        }
    }
}

#[cfg(test)]
mod test{
        use super::llvminterface;
        use super::LLVMContext;
        use super::LLVMModule;
        use super::Codegen;
        use torrentlearn_model::parse::Data;
        use torrentlearn_model::parse::Position;
        use torrentlearn_model::parse::Statement;
        use torrentlearn_model::parse::ConditionalStatement;
        use torrentlearn_model::parse::Statement::{SingleStatement};
        use torrentlearn_model::parse::ConditionalOperators;


        use std::u64;

        fn startLLVM() -> (LLVMContext,LLVMModule) {
                let mut context = llvminterface::initialize_llvm();
                let module = llvminterface::initialize_llvm_module(&mut context);
                (context,module)
        }
        #[test]
        fn test_value() {
            let mut array = [54;1000];
            let test = Data::Val(54);
            assert!(test_helper(Position::EndPos(0),test,"test".to_string(),&mut array));
            let test = Data::Val(u64::MAX);
            assert!(test_helper(Position::EndPos(0),test,"test".to_string(),&mut array));
            let test = Data::Val(u64::MIN);
            assert!(test_helper(Position::EndPos(0),test,"test".to_string(),&mut array));


        }
        #[test]
        fn test_position() {
            let array = [54;100];
            let test_val = Data::Val(54);
            let test = Position::EndPos(0);
            let test = Position::EndPos(10);
            let test = Position::EndPos(99);
            let test = Position::ContPos(Box::new(Position::EndPos(10)));
            let test = Position::ContPos(Box::new(Position::ContPos(Box::new(Position::EndPos(100)))));
            let test =Position::ContPos(Box::new(Position::ContPos(Box::new(Position::ContPos(Box::new(Position::EndPos(100)))))));
        }

        #[test]
        fn test_invalid_position() {
            //fix
        }

        fn test_helper(test_value: Position, expected_value: Data, test_name: String,test_pattern: &mut [u8]) -> bool {
            let (mut context,mut module) = startLLVM();
            let (mut function,mut arguments) = llvminterface::generate_function_proto(&mut context, &mut module,&test_name);
            let statement = ConditionalStatement(ConditionalOperators::Equals, test_value, expected_value).codegen(&mut context,&mut module,&mut arguments[0]);
            llvminterface::finalize_function(statement,function,&mut module);
            llvminterface::add_module_to_jit(&mut context,&mut module);
            llvminterface::dump_module_ir(&mut module);
            let function= llvminterface::get_pointer(&test_name).unwrap();
            function(test_pattern)
        }

        #[test]
        fn test_add() {


        }
        #[test]
        fn test_equality() {

        }
}
