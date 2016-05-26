//===----- KaleidoscopeJIT.h - A simple JIT for Kaleidoscope ----*- C++ -*-===//
//
//                     The LLVM Compiler Infrastructure
//
// This file is distributed under the University of Illinois Open Source
// License. See LICENSE.TXT for details.
//
//===----------------------------------------------------------------------===//
//
// Contains a simple JIT definition for use in the kaleidoscope tutorials.
//
//===----------------------------------------------------------------------===//

#ifndef LLVM_EXECUTIONENGINE_ORC_KALEIDOSCOPEJIT_H
#define LLVM_EXECUTIONENGINE_ORC_KALEIDOSCOPEJIT_H

#include "llvm/ExecutionEngine/ExecutionEngine.h"
#include "llvm/ExecutionEngine/RTDyldMemoryManager.h"
#include "llvm/ExecutionEngine/Orc/CompileUtils.h"
#include "llvm/ExecutionEngine/Orc/IRCompileLayer.h"
#include "llvm/ExecutionEngine/Orc/LambdaResolver.h"
#include "llvm/ExecutionEngine/Orc/ObjectLinkingLayer.h"
#include "llvm/IR/Mangler.h"
#include "llvm/Support/DynamicLibrary.h"
#include <unordered_map>

namespace llvm {
namespace orc {

class KaleidoscopeJIT {
public:
  typedef ObjectLinkingLayer<> ObjLayerT;
  typedef IRCompileLayer<ObjLayerT> CompileLayerT;
  typedef CompileLayerT::ModuleSetHandleT ModuleHandleT;

  KaleidoscopeJIT()
      : TM(EngineBuilder().selectTarget()), DL(TM->createDataLayout()),
        CompileLayer(ObjectLayer, SimpleCompiler(*TM)) {
    llvm::sys::DynamicLibrary::LoadLibraryPermanently(nullptr);
  }

  TargetMachine &getTargetMachine() { return *TM; }

  //Following the logic in the tutorial this should be a unique handle to the module
  unsigned addModule(Module* M) {
    // We need a memory manager to allocate memory and resolve symbols for this
    // new module. Create one that resolves symbols by looking back into the
    // JIT.
    auto Resolver = createLambdaResolver(
        [&](const std::string &Name) {
          if (auto Sym = findMangledSymbol(Name))
            return RuntimeDyld::SymbolInfo(Sym.getAddress(), Sym.getFlags());
          return RuntimeDyld::SymbolInfo(nullptr);
        },
        [](const std::string &S) { return nullptr; });
    auto H = CompileLayer.addModuleSet(singletonSet(std::move(M)),
                                       make_unique<SectionMemoryManager>(),
                                       std::move(Resolver));
    ModuleHandles[module_handle_count] = H;
    module_handle_count++;
    return module_handle_count-1;
  }

  void removeModule(unsigned module_handle_key) {
    if ( ModuleHandles.find(module_handle_key) == ModuleHandles.end()){
        //TODO: Refactor into some kind of cross language error handling (strings?)
        fprintf(stderr, "Couldn't locate llvm module handle, unrecoverable error, exiting\n");
        exit(EXIT_FAILURE);
    }
    CompileLayer.removeModuleSet(ModuleHandles[module_handle_key]);
    ModuleHandles.erase(module_handle_key);
    return;
  }

  JITSymbol findSymbol(const std::string Name) {
    return findMangledSymbol(mangle(Name));
  }

private:

  std::string mangle(const std::string &Name) {
    std::string MangledName;
    {
      raw_string_ostream MangledNameStream(MangledName);
      Mangler::getNameWithPrefix(MangledNameStream, Name, DL);
    }
    return MangledName;
  }

  template <typename T> static std::vector<T> singletonSet(T t) {
    std::vector<T> Vec;
    Vec.push_back(std::move(t));
    return Vec;
  }

  JITSymbol findMangledSymbol(const std::string &Name) {
    // Order of search is probably unecessary in this usecase, so we can use a map
    for (auto  const &key_value : ModuleHandles){
      if (auto Sym = CompileLayer.findSymbolIn(key_value.second, Name, true)){
          return Sym;
        }
    }
    fprintf(stderr, "Couldn't locate llvm function, unrecoverable error, exiting\n");
    exit(EXIT_FAILURE);
  }

  std::unique_ptr<TargetMachine> TM;
  const DataLayout DL;
  ObjLayerT ObjectLayer;
  CompileLayerT CompileLayer;
  std::unordered_map<unsigned, ModuleHandleT> ModuleHandles;
  unsigned module_handle_count = 0;
};

} // End namespace orc.
} // End namespace llvm

#endif // LLVM_EXECUTIONENGINE_ORC_KALEIDOSCOPEJIT_H
