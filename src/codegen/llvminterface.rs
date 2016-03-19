use super::LLVMValue;
use super::LLVMContext;
use super::LLVMModule;
use torrentlearn_model::parse::SingleOperators;
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

        pub fn extern_generate_cont_pos(context: *mut u8, array: *mut u8, val: u64)-> *mut u8;
        pub fn extern_generate_end_pos(context: *mut u8, array: *mut u8,val: u8)-> *mut u8;

        pub fn extern_generate_function_proto(context: *mut u8, module: *mut u8)-> FunctionProto;
        pub fn extern_finalize_function(fpm: *mut u8, function: *mut u8, body: *mut u8);
        pub fn extern_get_symbol(name: *const c_char) -> *mut u8;
        //pub fn extern_drop_value(value: *mut u8);
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
        LLVMValue(extern_generate_end_pos(context.context,array,position))
    }
}
pub fn generate_constant_pos(context: &mut LLVMContext, module: &mut LLVMModule, array: &mut LLVMValue) -> LLVMValue {
    unsafe {
        let &mut LLVMValue(array)= array;
        LLVMValue(extern_generate_cont_pos(context.context,array))
    }
}

pub fn generate_function_proto(context: &mut LLVMContext,module: &mut LLVMModule) -> (LLVMValue,Vec<LLVMValue>) {
    unsafe {
        let proto: FunctionProto = extern_generate_function_proto(context.context, module.module);
        let mut argument_vec = Vec::new();
        for value in proto.args.iter() {
            argument_vec.push(LLVMValue(*value));
        }
        return (LLVMValue(proto.proto),argument_vec)
    }
}
pub fn generate_single_statement(operator: SingleOperators, dest: LLVMValue,source:LLVMValue) -> LLVMValue
{
    unimplemented!()
}
pub fn finalize_function(body: LLVMValue, function: LLVMValue, module: &mut LLVMModule)
{
    let LLVMValue(body)= body;
    let LLVMValue(function)= function;
    unsafe{
        extern_finalize_function(module.function_pass_analyzer,body,function)
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

    static Operators: [SingleOperators; 1] = [Add];
    use parse::SingleOperators;

    #[test]
    fn null_and_sanity_checks_extern() {
        unsafe{
            let context = super::get_global_context();
            let jit = super::create_jit();
            let ir_builder = super::create_IRBuilder(context);
            assert!(!context.is_null());
            assert!(!jit.is_null());
            assert!(!ir_builder.is_null());

            let module = super::initialize_module(context,jit);
            assert!(!module.is_null());

            let fpm = super::initialize_pass_manager(module);
            assert!(!module.is_null());
        }


    }
}
