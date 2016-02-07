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

typedef struct FunctionProto_struct {
    void* proto;
    void* args[10];
} FunctionProto;


extern "C" void* get_global_context();
extern "C" void* create_jit();
extern "C" void* create_IRBuilder(void* context_void);
extern "C" void* initialize_module(void* context_void, void* jit_void);
extern "C" void* initialize_pass_manager(void* module_void);
extern "C" void* generate_constant(void* context_void, uint64_t value);
extern "C" void* extern_drop_value(void* value_void);
extern "C" FunctionProto extern_generate_function_proto(void* context_void, void* module_void);


/// CreateEntryBlockAlloca - Create an alloca instruction in the entry block of
/// the function.  This is used for mutable variables etc.
//FIXME: Using a different IR builder?
static AllocaInst *CreateEntryBlockAlloca(Function *TheFunction,
                                          const std::string &VarName) {
  IRBuilder<> TmpB(&TheFunction->getEntryBlock(),
                 TheFunction->getEntryBlock().begin());
  return TmpB.CreateAlloca(Type::getDoubleTy(getGlobalContext()), 0,
                           VarName.c_str());
}

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
FunctionProto extern_generate_function_proto(void* context_void, void* module_void)
{
    // Make the function type:  double(double,double) etc.
    std::vector<Type *> Doubles(Args.size(),
                              Type::getDoubleTy(getGlobalContext()));
    FunctionType *FT =
      FunctionType::get(Type::getDoubleTy(getGlobalContext()), Doubles, false);

    Function *F =
      Function::Create(FT, Function::ExternalLinkage, Name, TheModule.get());

    // Set names for all arguments.
    unsigned Idx = 0;
    for (auto &Arg : F->args())
        Arg.setName(Args[Idx++]);

    //TODO: Check how the ir builder managers insert points
    // Create a new basic block to start insertion into.
    BasicBlock *BB = BasicBlock::Create(getGlobalContext(), "entry", TheFunction);
    Builder.SetInsertPoint(BB);

    // Record the function arguments in the NamedValues map.
    NamedValues.clear();
    for (auto &Arg : TheFunction->args())
        NamedValues[Arg.getName()] = &Arg;
    return F;
}

FunctionProto extern_finalize_function(void* fpm, void* body, void* funtion)
{
    Builder.CreateRet(RetVal);

    // Validate the generated code, checking for consistency.
    verifyFunction(*TheFunction);

    // Run the optimizer on the function.
    TheFPM->run(*TheFunction);

    return TheFunction;

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
