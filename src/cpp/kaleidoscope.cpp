#include "llvm/ADT/STLExtras.h"
#include "llvm/Analysis/Passes.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Verifier.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Transforms/Scalar.h"
#include <cctype>
#include <string>
#include <vector>
#include <cstring>
#include "KaleidoscopeJIT.h"


using namespace llvm;
using namespace llvm::orc;

extern "C" void* get_global_context();
extern "C" void* create_jit();
extern "C" void* create_IRBuilder(void* context_void);
extern "C" void* initialize_module(void* context_void, void* jit_void);
extern "C" void* initialize_pass_manager(void* module_void);
extern "C" void* generate_constant(void* context_void, uint64_t value);
extern "C" void* test(void* context_void, void* ir_builder_void, uint64_t value);
extern "C" void* extern_drop_value(void* value_void);





void* get_global_context()
{
    return (void*) &getGlobalContext();
}

void* create_jit()
{
    //You will get null pointers if these aren't run and they should only be run
    //once by my understanding, but as we are only creating the JIT once that
    //should be fine
    InitializeNativeTarget();
    InitializeNativeTargetAsmPrinter();
    InitializeNativeTargetAsmParser();

    KaleidoscopeJIT* jit = new KaleidoscopeJIT();
    return static_cast<void*>(jit);
}

void* create_IRBuilder(void* context_void)
{
    LLVMContext* context = static_cast<LLVMContext*>(context_void);
    auto* ir_builder = new IRBuilder<>(*context);
    return (void*) ir_builder;
}
void* initialize_module(void* context_void, void* jit_void)
{
    KaleidoscopeJIT *jit= static_cast<KaleidoscopeJIT*>(jit_void);
    LLVMContext *context = static_cast<LLVMContext*>(context_void);

    Module* the_module = new Module("Torrent Learn JIT", *context);
    the_module->setDataLayout(jit->getTargetMachine().createDataLayout());
    return (void*) the_module;
}

void* initialize_pass_manager(void* module_void)
{
    Module *module = static_cast<Module*>(module_void);
    // Create a new pass manager attached to it.
    llvm::legacy::FunctionPassManager* fpm = new llvm::legacy::FunctionPassManager(module);

    // Do simple "peephole" optimizations and bit-twiddling optzns.
    fpm->add(createInstructionCombiningPass());
    // Reassociate expressions.
    fpm->add(createReassociatePass());
    // Eliminate Common SubExpressions.
    fpm->add(createGVNPass());
    // Simplify the control flow graph (deleting unreachable blocks, etc).
    fpm->add(createCFGSimplificationPass());

    fpm->doInitialization();

    return (void*) fpm;
}

void* extern_drop_value(void* value_void)
{
    Value *value = static_cast<Value*>(value_void);
    delete value;
}
void* generate_constant(void* context_void, uint64_t value)
{
    LLVMContext* context = static_cast<LLVMContext*>(context_void);
    return ConstantInt::get(*context, APInt(64,value));

}
