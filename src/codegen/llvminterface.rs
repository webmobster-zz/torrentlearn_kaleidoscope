use super::LLVMValue;
use super::LLVMContext;
use super::LLVMModule;

use parse::SingleOperators;

extern {

        pub fn get_global_context() -> *mut u8;
        pub fn create_jit() -> *mut u8;
        pub fn create_IRBuilder(context: *mut u8) -> *mut u8;
        pub fn initialize_module(context: *mut u8,jit: *mut u8)-> *mut u8;
        pub fn initialize_pass_manager(module: *mut u8)-> *mut u8;
}

fn initializeLLVM() -> LLVMContext
{
    let c; let j; let f;
    unsafe{
        // Get the global LLVM context, jit and ir builder
      c = get_global_context();
      j = create_jit();
      f = create_IRBuilder(c);
  }

  LLVMContext{context: c, kaleidoscope_jit:j, ir_builder: f}

}

fn initializeLLVMModule(context: LLVMContext) -> LLVMModule
{
    let m; let c;
    unsafe{
       // Create a new module for our function.
       m = initialize_module(context.context,context.kaleidoscope_jit);

       c = initialize_pass_manager(m);

   }
   LLVMModule{module: m, function_pass_analyzer: c}

}


pub fn generate_single_statement(operator: SingleOperators, dest: LLVMValue,source:LLVMValue) -> LLVMValue
{
    panic!("unimplemnted");
}
pub fn generate_constant(val: u64, context: &mut LLVMContext,module: &mut LLVMModule) -> LLVMValue
{
    panic!("unimplemnted");
}
pub fn generate_function(val: LLVMValue, context: &mut LLVMContext,module: &mut LLVMModule) -> LLVMValue
{
    panic!("unimplemnted");
}
pub fn load_array_cell(val: u8, context: &mut LLVMContext,module: &mut LLVMModule) -> LLVMValue
{
    panic!("unimplemnted");
}

#[cfg(test)]
mod tests {

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
