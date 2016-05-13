use torrentlearn_model::parse::ParseTree;
use torrentlearn_model::parse::Statement;
use torrentlearn_model::parse::ConditionalStatement;
use torrentlearn_model::parse::Statement::{SingleStatement};
use torrentlearn_model::parse::Data;
use torrentlearn_model::parse::Data::{Val,Pos};
use torrentlearn_model::parse::Position;
use torrentlearn_model::parse::Position::{ContPos,EndPos};
use torrentlearn_model::operator::DropHelper;
use std::sync::Arc;
use std::sync::Mutex;


use std::ffi::NulError;

pub mod llvminterface;

#[derive(Debug)]
pub enum CompileError {
    NulError(NulError)
}
impl From<NulError> for CompileError {
    fn from(err: NulError) -> CompileError {
        CompileError::NulError(err)
    }
}


pub trait Codegen {
    fn codegen(&self, context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule, fpm: &mut LLVMFpm, args: &mut LLVMValue) -> LLVMValue;
}

pub struct LLVMValue(*mut u8);

//Persistant across multiple modules
pub struct LLVMContext(*mut u8);
pub struct LLVMJit(*mut u8);
pub struct LLVMIrBuilder(*mut u8);

//Per module
pub struct LLVMFpm(*mut u8);
pub struct LLVMModule(*mut u8);

#[derive(Clone)]
pub struct CompiledModule{pub module: Arc<LLVMModule>,pub jit: Arc<Mutex<LLVMJit>>}

impl Drop for LLVMJit{
    fn drop(&mut self){
        //FIXME
        //panic!("unimplemented")
    }
}
impl Drop for LLVMIrBuilder{
    fn drop(&mut self){
        //FIXME
        //panic!("unimplemented")
    }
}

impl LLVMModule {
    pub fn with_context(context: &mut LLVMContext, jit: &mut LLVMJit) -> (LLVMModule, LLVMFpm) {
        llvminterface::initialize_llvm_module(context, jit)
    }
}

impl Drop for LLVMModule{
    fn drop(&mut self){
        //panic!("unimplemented")
    }
}

unsafe impl Send for CompiledModule{}

impl DropHelper for CompiledModule{
    fn trait_clone(&self) -> Box<DropHelper>{
        Box::new(self.clone()) as Box<DropHelper>
    }
}
//impl DropHelper for Box<CompiledModule>{}

//Use the module to keep track of uses for removal and dropping
pub fn compile(jit: &mut LLVMJit, context: &mut LLVMContext,ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule,fpm: &mut LLVMFpm, parsetree: &mut ParseTree, name: &str) -> Result<fn(&mut [u64]) -> bool,CompileError> {
    let (function,mut arg_temp) = llvminterface::generate_function_proto(context,ir_builder,module,name);
    let mut args = arg_temp.remove(0);
    let statement = parsetree.codegen(context,ir_builder,module,fpm,&mut args);
    llvminterface::finalize_function(statement,function,ir_builder,fpm);

    //FIXME: This assumes one function per module
    llvminterface::add_module_to_jit(jit,module);

    //FIXME: Text
    let pointer = try!(llvminterface::get_pointer(jit,"changeme"));
    return Ok(pointer)
}


impl Codegen for ParseTree {
    fn codegen(&self, context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule, fpm: &mut LLVMFpm, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &ParseTree::EndSingle(ref statement) => {
                statement.codegen(context,ir_builder,module,fpm,args)
            },
            &ParseTree::EndConditional(ref statement) =>  {
                statement.codegen(context,ir_builder,module,fpm,args)
            },
            &ParseTree::Continuation(ref continuation, ref statement) =>{
                //discard previous result
                statement.codegen(context,ir_builder,module,fpm,args);
                continuation.codegen(context,ir_builder,module,fpm,args)
            }
        }
    }
}

impl Codegen for Statement {
    fn codegen(&self, context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule, fpm: &mut LLVMFpm, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &SingleStatement(ref operator,ref pos,ref data) =>{
                llvminterface::generate_single_statement(*operator,pos.codegen(context,ir_builder,module,fpm,args),data.codegen(context,ir_builder,module,fpm,args))
            }
        }
    }
}
impl Codegen for ConditionalStatement {
    fn codegen(&self, context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule, fpm: &mut LLVMFpm, args: &mut LLVMValue) -> LLVMValue {
        let ConditionalStatement(ref operator,ref pos,ref data) = *self;
        let dest = pos.codegen(context,ir_builder,module,fpm,args);
        let source = data.codegen(context,ir_builder,module,fpm,args);
        llvminterface::generate_conditional_statement(ir_builder,*operator,dest,source)

    }
}

impl Codegen for Data {
    fn codegen(&self, context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule, fpm: &mut LLVMFpm, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &Val(val) => llvminterface::generate_constant_val(context,val),
            &Pos(ref position) => position.codegen(context,ir_builder,module,fpm,args)
        }
    }
}
impl Codegen for Position {
    fn codegen(&self, context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule, fpm: &mut LLVMFpm, args: &mut LLVMValue) -> LLVMValue {
        match self {
            &ContPos(ref next) => {
                let array_index = next.codegen(context,ir_builder,module,fpm,args);
                llvminterface::generate_cont_pos(context,ir_builder, args,array_index)
            },
            &EndPos(val) => llvminterface::generate_end_pos(context,ir_builder,args,val)
        }
    }
}

#[cfg(test)]
mod test{
        use super::llvminterface;
        use super::LLVMContext;
        use super::LLVMModule;
        use super::LLVMJit;
        use super::LLVMFpm;
        use super::LLVMIrBuilder;


        use super::Codegen;
        use torrentlearn_model::parse::Data;
        use torrentlearn_model::parse::Position;
        use torrentlearn_model::parse::Statement;
        use torrentlearn_model::parse::ConditionalStatement;
        use torrentlearn_model::parse::Statement::{SingleStatement};
        use torrentlearn_model::parse::ConditionalOperators;
        use std::mem;



        use std::u64;

        fn startLLVM() -> (LLVMContext,LLVMJit,LLVMIrBuilder,LLVMModule,LLVMFpm ) {
                let (mut context,mut jit,irbuilder) = llvminterface::initialize_llvm();
                let (module,fpm) = llvminterface::initialize_llvm_module(&mut context,&mut jit);
                (context,jit,irbuilder,module,fpm)
        }

        #[test]
        fn test_threading() {
            let mut array = [54;1000];
            let test = Data::Val(54);
            assert!(test_helper(Position::EndPos(0),test,"test".to_string(),&mut array));
            let test = Data::Val(u64::MAX);
            assert!(test_helper(Position::EndPos(0),test,"test".to_string(),&mut array));
            let test = Data::Val(u64::MIN);
            assert!(test_helper(Position::EndPos(0),test,"test".to_string(),&mut array));
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
            let array: [u64;100] = [54;100];

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

        fn test_helper(test_value: Position, expected_value: Data, test_name: String,test_pattern: &mut [u64]) -> bool {
            let (mut context,mut jit,mut ir_builder,mut module,mut fpm) = startLLVM();
            let (mut function,mut arguments) = llvminterface::generate_function_proto(&mut context,&mut ir_builder, &mut module,&test_name);
            let statement = ConditionalStatement(ConditionalOperators::Equals, test_value, expected_value).codegen(&mut context,&mut ir_builder, &mut module,&mut fpm,&mut arguments[0]);
            llvminterface::finalize_function(statement,function,&mut ir_builder, &mut fpm);
            llvminterface::dump_module_ir(&mut module);
            llvminterface::add_module_to_jit(&mut jit,&mut module);
            let function= llvminterface::get_pointer(&mut jit,&test_name).unwrap();
            function(test_pattern)
        }

        #[test]
        fn test_add() {


        }
        #[test]
        fn test_equality() {

        }
}
