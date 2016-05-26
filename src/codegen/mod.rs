use torrentlearn_model::parse::ParseTree;
use torrentlearn_model::parse::Statement;
use torrentlearn_model::parse::ConditionalStatement;
use torrentlearn_model::parse::Statement::{SingleStatement};
use torrentlearn_model::parse::Data;
use torrentlearn_model::parse::Data::{Val,Pos};
use torrentlearn_model::parse::Position;
use torrentlearn_model::parse::Position::{ContPos,EndPos};
use torrentlearn_model::operator::DropHelper;
use std::sync::{Arc,Mutex};
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
//Unique handle to module in the JIT
pub struct LLVMModuleHandle(usize);


impl Drop for LLVMJit{
    fn drop(&mut self){
        llvminterface::drop_jit(self)
    }
}
impl Drop for LLVMIrBuilder{
    fn drop(&mut self){
        llvminterface::drop_irbuilder(self)
    }
}

impl LLVMModule {
    pub fn with_context(context: &mut LLVMContext, jit: &mut LLVMJit) -> (LLVMModule, LLVMFpm) {
        llvminterface::initialize_llvm_module(context, jit)
    }
}

impl Drop for LLVMModule{
    //This module should never be dropped from rust side, only way to get rid of it (for now) is
    //adding it to the jit, if it is required a drop implementation that calls the c++ will be created
    fn drop(&mut self){
        unreachable!()
    }
}
impl Drop for LLVMFpm{
    fn drop(&mut self){
        llvminterface::drop_fpm(self)
    }
}
impl Drop for LLVMModuleHandle{
    //This value needs to be cleaned up by the jit and doesn't make sense to be dropped alone
    fn drop(&mut self){
        unreachable!()
    }
}

#[derive(Clone)]
//Option to stop recursive dropping
pub struct CompiledModule{handle: Option<Arc<LLVMModuleHandle>>,jit: Arc<Mutex<LLVMJit>>}

unsafe impl Send for CompiledModule{}


impl CompiledModule{
    pub fn new(handle: LLVMModuleHandle, jit: Arc<Mutex<LLVMJit>>) -> CompiledModule{
        CompiledModule{handle: Some(Arc::new(handle)),jit: jit}
    }
}
impl DropHelper for CompiledModule{
    fn trait_clone(&self) -> Box<DropHelper>{
        Box::new(self.clone()) as Box<DropHelper>
    }
}
impl Drop for CompiledModule{
    fn drop(&mut self){
        //Swap the module for a None, then unwrap it as the optional is only used for the
        //destructor it should never be None
        match Arc::try_unwrap(self.handle.take().unwrap()){
            Ok(module) =>{
                let mut jit = match self.jit.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => { error!("Jit lock poisoned, attempting to continue"); poisoned.into_inner()}
                };
                llvminterface::remove_module_from_jit(&mut jit, module);
            },
            Err(_) =>{/*do nothing as it is not the last reference to the module, should be safe to drop the arc*/}
        }
    }
}

//Use the module to keep track of uses for removal and dropping
pub fn compile(jit: &mut LLVMJit, context: &mut LLVMContext,ir_builder: &mut LLVMIrBuilder, mut module: LLVMModule,fpm: &mut LLVMFpm, parsetree: &mut ParseTree, name: &str) -> Result<(fn(&mut [u64]) -> bool,LLVMModuleHandle),CompileError> {
    let (function,mut arg_temp) = llvminterface::generate_function_proto(context,ir_builder,&mut module,name);
    let mut args = arg_temp.remove(0);
    let statement = parsetree.codegen(context,ir_builder,&mut module,fpm,&mut args);
    llvminterface::finalize_function(statement,function,ir_builder,fpm);

    //FIXME: This assumes one function per module, look into batching them up
    let handle=llvminterface::add_module_to_jit(jit,module);

    //FIXME: Text
    let pointer = try!(llvminterface::get_pointer(jit,"changeme"));
    return Ok((pointer,handle))
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
unsafe impl Send for LLVMJit{}
#[cfg(test)]
unsafe impl Send for LLVMContext{}
#[cfg(test)]
unsafe impl Send for LLVMIrBuilder{}


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
        use std::sync::{Arc,Mutex};
        use std::thread;

        lazy_static! {
            static ref jjit: Arc<Mutex<(LLVMContext,LLVMJit,LLVMIrBuilder)>> = {
                let (mut context,mut jit,irbuilder) = llvminterface::initialize_llvm();
                Arc::new(Mutex::new((context,jit,irbuilder)))
            };
        }

        fn startLLVM(context: &mut LLVMContext, jit: &mut LLVMJit) -> (LLVMModule,LLVMFpm ) {
            let (module,fpm) = llvminterface::initialize_llvm_module(context,jit);
            (module,fpm)
        }
/*
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
            assert!(test_helper(Position::EndPos(0),test,"test2".to_string(),&mut array));
            let test = Data::Val(u64::MIN);
            assert!(test_helper(Position::EndPos(0),test,"test3".to_string(),&mut array));
        }
        /*#[test]
        fn test_position() {
            let array: [u64;100] = [54;100];

            let test_val = Data::Val(54);
            let test = Position::EndPos(0);
            let test = Position::EndPos(10);
            let test = Position::EndPos(99);
            let test = Position::ContPos(Box::new(Position::EndPos(10)));
            let test = Position::ContPos(Box::new(Position::ContPos(Box::new(Position::EndPos(100)))));
            let test =Position::ContPos(Box::new(Position::ContPos(Box::new(Position::ContPos(Box::new(Position::EndPos(100)))))));
        }*/
*/
        fn test_helper(test_value: Position, expected_value: Data, test_name: String,test_pattern: &mut [u64]) -> bool {
            let (ref mut context,ref mut jit, ref mut ir_builder) = *(jjit.lock().unwrap());
            let (mut module,mut fpm) = startLLVM(context,jit);
            let (mut function,mut arguments) = llvminterface::generate_function_proto(context,ir_builder, &mut module,&test_name);
            let statement = ConditionalStatement(ConditionalOperators::Equals, test_value, expected_value).codegen(context,ir_builder, &mut module,&mut fpm,&mut arguments[0]);
            llvminterface::finalize_function(statement,function,ir_builder, &mut fpm);
            llvminterface::dump_module_ir(&mut module);
            let handle = llvminterface::add_module_to_jit(jit,module);
            let function= llvminterface::get_pointer(jit,&test_name).unwrap();
            let result = function(test_pattern);
            llvminterface::remove_module_from_jit(jit, handle);
            result
        }
        #[test]
        fn test_sigsev_on_panic_issue(){
            let (ref mut context,ref mut jit, ref mut ir_builder) = *(jjit.lock().unwrap());
            let (mut module,mut fpm) = startLLVM(context,jit);
            let (mut function,mut arguments) = llvminterface::generate_function_proto(context,ir_builder, &mut module,&"test");
            let statement = Data::Val(54).codegen(context,ir_builder, &mut module,&mut fpm,&mut arguments[0]);
            llvminterface::finalize_function(statement,function,ir_builder, &mut fpm);
            llvminterface::dump_module_ir(&mut module);
            let handle = llvminterface::add_module_to_jit(jit,module);
            {
                let function= llvminterface::get_pointer(jit,&"test").unwrap();
            }
            llvminterface::remove_module_from_jit(jit, handle);
            //panic!("");
        }
}
