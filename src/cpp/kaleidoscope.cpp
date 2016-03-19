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

typedef struct FunctionProto {
    void* proto;
    void* args[10];
} FunctionProto;

typedef bool (*FunctionPtr)(uint64_t);



extern "C" void* extern_get_global_context();
extern "C" void* extern_create_jit();
extern "C" void* extern_create_IRBuilder(void* context_void);
extern "C" void* extern_initialize_module(void* context_void, void* jit_void);
extern "C" void* extern_initialize_pass_manager(void* module_void);
extern "C" void* extern_generate_constant(void* context_void, uint64_t value);
extern "C" void* extern_generate_end_pos(void* context_void,void* builder_void, void* array_void,uint64_t values);
extern "C" void* extern_generate_cont_pos(void* context_void,void* builder_void, void* array_void, void* access_value_void);
extern "C" void* extern_drop_value(void* value_void);
extern "C" FunctionProto extern_generate_function_proto(void* context_void, void* module_void,void* builder_void, char* name_c_str);

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

void* extern_get_global_context() {
    return (void*) &getGlobalContext();
}

void* extern_create_jit() {
    //You will get null pointers if these aren't run and they should only be run
    //once by my understanding, but as we are only creating the JIT once that
    //should be fine
    InitializeNativeTarget();
    InitializeNativeTargetAsmPrinter();
    InitializeNativeTargetAsmParser();
    KaleidoscopeJIT* jit = new KaleidoscopeJIT();
    return static_cast<void*>(jit);
}

void* extern_create_IRBuilder(void* context_void)
{
    LLVMContext* context = static_cast<LLVMContext*>(context_void);
    auto* ir_builder = new IRBuilder<>(*context);
    return (void*) ir_builder;
}
void* extern_initialize_module(void* context_void, void* jit_void)
{
    KaleidoscopeJIT *jit= static_cast<KaleidoscopeJIT*>(jit_void);
    LLVMContext *context = static_cast<LLVMContext*>(context_void);

    Module* the_module = new Module("Torrent Learn JIT", *context);
    the_module->setDataLayout(jit->getTargetMachine().createDataLayout());
    return (void*) the_module;
}

void* extern_initialize_pass_manager(void* module_void)
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

void* extern_generate_constant(void* context_void, uint64_t value) {
    LLVMContext* context = static_cast<LLVMContext*>(context_void);
    return ConstantInt::get(*context, APInt(64,value));
}
void* extern_generate_end_pos(void* context_void,void* builder_void, void* array_void,uint64_t index) {
    IRBuilder<>* builder = static_cast<IRBuilder<>*>(builder_void);
    LLVMContext* context = static_cast<LLVMContext*>(context_void);
    Value array = static_cast<Value*>(array_void);
    Value* indexList[1];
    indexList[0] = ConstantInt::get(*context, APInt(64,index));
    return (void*) builder.CreateGEP(array, idxList);
}
void* extern_generate_cont_pos(void* builder_void, void* array_void, void* access_value_void) {
    IRBuilder<>* builder = static_cast<IRBuilder<>*>(builder_void);
    LLVMContext* context = static_cast<LLVMContext*>(context_void);
    Value array = static_cast<Value*>(array_void);
    Value access_value_pointer = static_cast<Value*>(access_value_void);
    Value* access_value = builder.CreateLoad(access_value_pointer)
    Value* indexList[1];
    indexList[0] = access_value;
    return (void*) builder.CreateGEP(array, idxList);
}
FunctionProto extern_generate_function_proto(void* context_void, void* module_void,void* builder_void, char* name_c_str) {
    LLVMContext *context = static_cast<LLVMContext*>(context_void);
    Module *module = static_cast<Module*>(module_void);
    IRBuilder<> *builder = static_cast<IRBuilder<>*>(builder_void);
    std::string Name = name_c_str;

    //FIXME: Make this more visible
    // Make the function type:  double(double,double) etc.
    std::vector<Type *> argument_list(1,
                              Type::getInt64PtrTy(*context));
    FunctionType *function_type =
      FunctionType::get(Type::getInt1Ty(*context), argument_list, false);
    Function *function =
      Function::Create(function_type, Function::ExternalLinkage, Name, module);

    // Set names for all arguments.
    //FIXME: Do we need this?
    /*
    unsigned Idx = 0;
    for (auto &Arg : F->args())
        Arg.setName(Args[Idx++]);
    */

    //TODO: Check how the ir builder managers insert points, make this less of a global thing that
    //modifys multiple things
    // Create a new basic block to start insertion into.
    BasicBlock *BB = BasicBlock::Create(*context, "entry", function);
    builder->SetInsertPoint(BB);

    void* args[10];
    unsigned idx = 0;
    for (Argument &Arg : function->args()) {

        // Create an alloca for this variable.
        AllocaInst *Alloca = CreateEntryBlockAlloca(TheFunction, Arg.getName());

        // Store the initial value into the alloca.
        Builder.CreateStore(&Arg, Alloca);
        args[idx] = (void*) &Alloca;
        idx++;
    }
    FunctionProto function_proto_struct;
    function_proto_struct = {(void*) function, args};
    return function_proto_struct;
}

void extern_finalize_function(void* fpm_void, void* body_void, void* function_void) {
    llvm::legacy::FunctionPassManager* fpm =  static_cast<IRBuilder<>*>(fpm_void);

    IRBuilder<> *builder = static_cast<IRBuilder<>*>(builder_void);
    Value *body = static_cast<Value*>(body_void);
    Function *function_void = static_cast<Function*>(function_void);

    builder.CreateRet(body);

    // Validate the generated code, checking for consistency.
    verifyFunction(*function);

    // Run the optimizer on the function.
    fpm->run(*function);
    TheJIT->addModule(std::move(TheModule));
    return;

}
FunctionPtr extern_get_symbol(const char* name) {
    // Search the JIT for the __anon_expr symbol.
     auto ExprSymbol = jit->findSymbol(name);
     assert(ExprSymbol && "Function not found");

     // Get the symbol's address and cast it to the right type (takes no
     // arguments, returns a double) so we can call it as a native function.
     return (FunctionPtr)(intptr_t)ExprSymbol.getAddress();
}

void* extern_drop_value(void* value_void)
{
    Value *value = static_cast<Value*>(value_void);
    delete value;
}
