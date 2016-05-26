use super::LLVMValue;
use super::LLVMContext;
use super::LLVMJit;
use super::LLVMIrBuilder;
use super::LLVMModule;
use super::LLVMModuleHandle;
use super::LLVMFpm;
use torrentlearn_model::parse::SingleOperators;
use torrentlearn_model::parse::ConditionalOperators;
use std::os::raw::c_char;
use std::ffi::CString;
use std::ffi::NulError;
use std::mem;

extern {

        pub fn extern_get_global_context() -> *mut u8;
        pub fn extern_create_jit() -> *mut u8;
        pub fn extern_create_IRBuilder(context: *mut u8) -> *mut u8;
        pub fn extern_initialize_module(context: *mut u8,jit: *mut u8)-> *mut u8;
        pub fn extern_initialize_pass_manager(module: *mut u8)-> *mut u8;

        pub fn extern_drop_jit(jit: *mut u8);
        pub fn extern_drop_fpm(fpm: *mut u8);
        pub fn extern_drop_ir_builder(ir_builder: *mut u8);

        pub fn extern_generate_constant(context: *mut u8, val: u64)-> *mut u8;
        pub fn extern_generate_end_pos(context: *mut u8,builder: *mut u8, array: *mut u8, index: u64)-> *mut u8;
        pub fn extern_generate_cont_pos(builder: *mut u8, array: *mut u8, array_index_pointer: *mut u8)-> *mut u8;
        pub fn extern_generate_function_proto(context: *mut u8, module: *mut u8, builder: *mut u8, name: *const c_char)-> FunctionProto;
        pub fn extern_finalize_function(builder: *mut u8, fpm: *mut u8, function: *mut u8, body: *mut u8) -> *mut u8;

        pub fn extern_get_symbol(jit: *mut u8, name: *const c_char) -> *mut u8;
        pub fn extern_add_module_to_jit(context: *mut u8,module: *mut u8) -> usize;
        pub fn extern_remove_module_from_jit(context: *mut u8,handle: usize);
        pub fn extern_dump_module_ir(module: *mut u8);

        //Operators
        pub fn extern_create_equals_statement(builder: *mut u8,source: *mut u8,destination: *mut u8) -> *mut u8;
}


#[repr(C)]
pub struct FunctionProto {
    proto: *mut u8,
    args: [*mut u8;10]
}

/* Constructors */
pub fn initialize_llvm() -> (LLVMContext,LLVMJit,LLVMIrBuilder) {
    let c; let j; let f;
    unsafe{
        // Get the global LLVM context, jit and ir builder
      c = extern_get_global_context();
      f = extern_create_IRBuilder(c);

       j = extern_create_jit();
   }
  (LLVMContext(c),LLVMJit(j),LLVMIrBuilder(f))
}

pub fn initialize_llvm_module(context: &mut LLVMContext, jit: &mut LLVMJit) -> (LLVMModule,LLVMFpm) {
    let m; let c;
    let &mut LLVMContext(context)= context; let &mut LLVMJit(jit)= jit;
    unsafe{
       // Create a new module for our function.
       m = extern_initialize_module(context,jit);
       c = extern_initialize_pass_manager(m);
   }
   (LLVMModule(m), LLVMFpm(c))
}

/* Destructors */
pub fn drop_jit(jit: &mut LLVMJit){
    let &mut LLVMJit(jit)= jit;
    unsafe{
        extern_drop_jit(jit)
    }
}
pub fn drop_fpm(fpm: &mut LLVMFpm){
    let &mut LLVMFpm(fpm)= fpm;
    unsafe{
        extern_drop_fpm(fpm)
    }
}
pub fn drop_irbuilder(ir_builder: &mut LLVMIrBuilder){
    let &mut LLVMIrBuilder(ir_builder)= ir_builder;
    unsafe{
        extern_drop_ir_builder(ir_builder)
    }
}

/* Code Generators */
pub fn generate_constant_val(context: &mut LLVMContext, val: u64) -> LLVMValue {
    let &mut LLVMContext(context)= context;
    unsafe{
        LLVMValue(extern_generate_constant(context,val))
    }
}
pub fn generate_end_pos(context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, array: &mut LLVMValue, position: u64) -> LLVMValue {
    let &mut LLVMContext(context)= context;
    let &mut LLVMIrBuilder(ir_builder)= ir_builder;
    unsafe{
        let &mut LLVMValue(array)= array;
        LLVMValue(extern_generate_end_pos(context,ir_builder,array,position))
    }
}
pub fn generate_cont_pos(context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, array: &mut LLVMValue, index: LLVMValue) -> LLVMValue {
    let &mut LLVMIrBuilder(ir_builder)= ir_builder;
    let &mut LLVMValue(array)= array;
    let LLVMValue(index)= index;
    unsafe {
        LLVMValue(extern_generate_cont_pos(ir_builder,array,index))
    }
}

pub fn generate_function_proto(context: &mut LLVMContext, ir_builder: &mut LLVMIrBuilder, module: &mut LLVMModule,name: &str) -> (LLVMValue,Vec<LLVMValue>) {
    let &mut LLVMContext(context)= context;
    let &mut LLVMModule(module)= module;
    let &mut LLVMIrBuilder(ir_builder)= ir_builder;
    let proto: FunctionProto;
    unsafe {
        //FIXME
        let name = (CString::new(name).unwrap()).as_ptr();
        proto = extern_generate_function_proto(context, module,ir_builder,name);

    }
    let mut argument_vec = Vec::new();
    for value in proto.args.iter() {
        argument_vec.push(LLVMValue(*value));
    }
    return (LLVMValue(proto.proto),argument_vec)
}
pub fn generate_conditional_statement(ir_builder: &mut LLVMIrBuilder, operator: ConditionalOperators, destination: LLVMValue,source:LLVMValue) -> LLVMValue {
    let LLVMValue(source)= source;
    let LLVMValue(destination)= destination;
    let &mut LLVMIrBuilder(ir_builder)= ir_builder;
    unsafe {
        LLVMValue(match operator {
            ConditionalOperators::Equals => extern_create_equals_statement(ir_builder,source,destination)
        })
    }
}
pub fn generate_single_statement(operator: SingleOperators, dest: LLVMValue,source:LLVMValue) -> LLVMValue {
    unimplemented!()
}
pub fn finalize_function(body: LLVMValue, function: LLVMValue, ir_builder: &mut LLVMIrBuilder,  fpm: &mut LLVMFpm) -> LLVMValue {
    let LLVMValue(body)= body;
    let LLVMValue(function)= function;
    let &mut LLVMIrBuilder(ir_builder)= ir_builder;
    let &mut LLVMFpm(fpm)= fpm;
    unsafe{
        LLVMValue(extern_finalize_function(ir_builder,fpm,body,function))
    }
}

/* Other and Utils */

//To keep with the same behaviour as the kalidoscope tutorial move the module into the JIT,
//don't send a pointer to it
pub fn add_module_to_jit(jit: &mut LLVMJit, module: LLVMModule) -> LLVMModuleHandle{
    let LLVMModule(module_inner)= module;
    //Don't let the module be cleaned up as we have invalidated the drop
    mem::forget(module);
    let &mut LLVMJit(jit)= jit;
    unsafe{
        return LLVMModuleHandle(extern_add_module_to_jit(jit,module_inner))
    }
}

pub fn remove_module_from_jit(jit: &mut LLVMJit, handle: LLVMModuleHandle) {
    let LLVMModuleHandle(handle_inner)= handle;
    //Don't let the handle be cleaned up as we have invalidated the drop
    mem::forget(handle);
    let &mut LLVMJit(jit)= jit;
    unsafe{
        extern_remove_module_from_jit(jit,handle_inner)
    }
}


#[allow(dead_code)]
//Used for debugging and tests
pub fn dump_module_ir(module: &mut LLVMModule){
    let &mut LLVMModule(module)= module;
    unsafe{
        extern_dump_module_ir(module);
    }
}

pub fn get_pointer(jit: &mut LLVMJit, name: &str) ->  Result<fn(&mut [u64]) -> bool,NulError>{
    let &mut LLVMJit(jit)= jit;
    unsafe{
        let name = (CString::new(name).unwrap()).as_ptr();
        let symbol =extern_get_symbol(jit,name);
        Ok(mem::transmute::<*mut u8,fn(&mut [u64]) -> bool>(symbol))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn null_and_sanity_checks_extern() {
        unsafe{
            let context = super::extern_get_global_context();
            let jit = super::extern_create_jit();
            let ir_builder = super::extern_create_IRBuilder(context);
            assert!(!context.is_null());
            assert!(!jit.is_null());
            assert!(!ir_builder.is_null());

            let module = super::extern_initialize_module(context,jit);
            assert!(!module.is_null());

            let fpm = super::extern_initialize_pass_manager(module);
            assert!(!module.is_null());
        }
    }
}
