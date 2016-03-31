use super::LLVMValue;
use super::LLVMContext;
use super::LLVMModule;
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
        pub fn extern_generate_constant(context: *mut u8, val: u64)-> *mut u8;

        pub fn extern_generate_end_pos(context: *mut u8,builder: *mut u8, array: *mut u8, index: u64)-> *mut u8;
        pub fn extern_generate_cont_pos(builder: *mut u8, array: *mut u8, array_index_pointer: *mut u8)-> *mut u8;

        pub fn extern_generate_function_proto(context: *mut u8, module: *mut u8, builder: *mut u8, name: *const c_char)-> FunctionProto;
        pub fn extern_finalize_function(fpm: *mut u8, function: *mut u8, body: *mut u8) -> *mut u8;
        pub fn extern_get_symbol(name: *const c_char) -> *mut u8;
        pub fn extern_add_module_to_jit(context: *mut u8,module: *mut u8);
        pub fn extern_dump_module_ir(module: *mut u8);


        //pub fn extern_drop_value(value: *mut u8);

        //Operators
}

#[repr(C)]
pub struct FunctionProto {
    proto: *mut u8,
    args: [*mut u8;10]
}
pub fn initialize_llvm() -> LLVMContext {
    let c; let j; let f;
    unsafe{
        // Get the global LLVM context, jit and ir builder
      c = extern_get_global_context();
      j = extern_create_jit();
      f = extern_create_IRBuilder(c);
  }
  LLVMContext{context: c, kaleidoscope_jit:j, ir_builder:f}
}

pub fn initialize_llvm_module(context: &mut LLVMContext) -> LLVMModule{
    let m; let c;
    unsafe{
       // Create a new module for our function.
       m = extern_initialize_module(context.context,context.kaleidoscope_jit);
       c = extern_initialize_pass_manager(m);
   }
   LLVMModule{module: m, function_pass_analyzer: c}
}
pub fn generate_constant_val(context: &mut LLVMContext, val: u64) -> LLVMValue {
    unsafe{
        LLVMValue(extern_generate_constant(context.context,val))
    }
}
pub fn generate_end_pos(context: &mut LLVMContext, array: &mut LLVMValue, position: u64) -> LLVMValue {
    unsafe{
        let &mut LLVMValue(array)= array;
        LLVMValue(extern_generate_end_pos(context.context,context.ir_builder,array,position))
    }
}
pub fn generate_cont_pos(context: &mut LLVMContext,array: &mut LLVMValue, index: LLVMValue) -> LLVMValue {
    unsafe {
        let &mut LLVMValue(array)= array;
        let LLVMValue(index)= index;
        LLVMValue(extern_generate_cont_pos(context.ir_builder,array,index))
    }
}

pub fn generate_function_proto(context: &mut LLVMContext,module: &mut LLVMModule,name: &str) -> (LLVMValue,Vec<LLVMValue>) {
    unsafe {
        //FIXME
        let name = (CString::new(name).unwrap()).as_ptr();
        let proto: FunctionProto = extern_generate_function_proto(context.context, module.module,context.ir_builder,name);
        let mut argument_vec = Vec::new();
        for value in proto.args.iter() {
            argument_vec.push(LLVMValue(*value));
        }
        return (LLVMValue(proto.proto),argument_vec)
    }
}
pub fn generate_conditional_statement(operator: ConditionalOperators, dest: LLVMValue,source:LLVMValue) -> LLVMValue {
    unimplemented!()
}
pub fn generate_single_statement(operator: SingleOperators, dest: LLVMValue,source:LLVMValue) -> LLVMValue {
    unimplemented!()
}
pub fn finalize_function(body: LLVMValue, function: LLVMValue, module: &mut LLVMModule) -> LLVMValue {
    let LLVMValue(body)= body;
    let LLVMValue(function)= function;
    unsafe{
        LLVMValue(extern_finalize_function(module.function_pass_analyzer,body,function))
    }
}
pub fn add_module_to_jit(context: &mut LLVMContext, module: &mut LLVMModule) {
    unsafe{
        extern_add_module_to_jit(context.kaleidoscope_jit,module.module)
    }
}

#[allow(dead_code)]
//Used for debugging and tests
pub fn dump_module_ir(module: &mut LLVMModule){
    unsafe{
        extern_dump_module_ir(module.module);
    }
}

pub fn get_pointer(name: &str) ->  Result<fn(&mut [u8]) -> bool,NulError>{
    unsafe{
        let name = try!(CString::new(name)).as_ptr();
        Ok(mem::transmute::<*mut u8,fn(&mut [u8]) -> bool>(extern_get_symbol(name)))
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
