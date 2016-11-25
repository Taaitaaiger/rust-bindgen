#include "clang/Frontend/FrontendPluginRegistry.h"
#include "clang/AST/AST.h"
#include "clang/AST/ASTConsumer.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include "clang/Frontend/CompilerInstance.h"

#include "RustBindgen.h"
#include "BindgenContext.h"
#include "BindgenItem.h"

namespace bindgen {

using clang::ASTConsumer;
using clang::CompilerInstance;
using clang::PluginASTAction;
using clang::dyn_cast;
using clang::CXXRecordDecl;
using clang::RecordDecl;

void IRGenerator::HandleTranslationUnit(clang::ASTContext& astContext) {
  BindgenContext ctx(astContext, m_compilerInstance);
}

class BindgenAction : public PluginASTAction {
protected:
  std::unique_ptr<ASTConsumer> CreateASTConsumer(CompilerInstance& compiler,
                                                 llvm::StringRef) override {
    return std::make_unique<IRGenerator>(compiler);
  }

  bool ParseArgs(const CompilerInstance&,
                 const std::vector<std::string>&) override {
    return true;
  }
};

}  // namespace bindgen

static clang::FrontendPluginRegistry::Add<bindgen::BindgenAction> X(
    "rust-bindgen", "generate rust bindgen's IR");
