use super::item::{Item, ItemId};
use super::context::BindgenContext;
use super::ty::TypeKind;
use super::int::IntKind;
use parse::{ClangItemParser, ClangSubItemParser, ParseResult, ParseError};
use clang;

#[derive(Debug)]
pub struct Var {
    /// The name of the variable.
    name: String,
    /// The mangled name of the variable.
    mangled_name: Option<String>,
    /// The type of the variable.
    ty: ItemId,
    /// TODO: support non-integer constants?
    /// The integer value of the variable.
    val: Option<i64>,
    /// Whether this variable is const.
    is_const: bool,
}

impl Var {
    pub fn new(name: String,
               mangled: Option<String>,
               ty: ItemId,
               val: Option<i64>,
               is_const: bool) -> Var {
        assert!(!name.is_empty());
        Var {
            name: name,
            mangled_name: mangled,
            ty: ty,
            val: val,
            is_const: is_const,
        }
    }

    pub fn is_const(&self) -> bool {
        self.is_const
    }

    pub fn val(&self) -> Option<i64> {
        self.val
    }

    pub fn ty(&self) -> ItemId {
        self.ty
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mangled_name(&self) -> Option<&str> {
        self.mangled_name.as_ref().map(|n| &**n)
    }
}

impl ClangSubItemParser for Var {
    fn parse(cursor: clang::Cursor,
             context: &mut BindgenContext) -> Result<ParseResult<Self>, ParseError> {
        use clangll::*;
        match cursor.kind() {
            CXCursor_MacroDefinition => {
                let value = match parse_int_literal_tokens(&cursor, context.translation_unit(), 1) {
                    None => return Err(ParseError::Continue),
                    Some(v) => v,
                };

                let name = cursor.spelling();
                if name.is_empty() {
                    warn!("Empty macro name?");
                    return Err(ParseError::Continue);
                }

                if context.parsed_macro(&name) {
                    warn!("Duplicated macro definition: {}", name);
                    return Err(ParseError::Continue);
                }
                context.note_parsed_macro(name.clone());

                let ty = if value.abs() > u32::max_value() as i64  {
                    Item::builtin_type(TypeKind::Int(IntKind::ULongLong), true, context)
                } else {
                    Item::builtin_type(TypeKind::Int(IntKind::UInt), true, context)
                };

                Ok(ParseResult::New(Var::new(name, None, ty, Some(value), true), Some(cursor)))
            }
            CXCursor_VarDecl => {
                println!("Vardecl");
                let name = cursor.spelling();
                if name.is_empty() {
                    warn!("Empty constant name?");
                    return Err(ParseError::Continue);
                }

                let ty = cursor.cur_type();

                // XXX this is redundant, remove!
                let is_const = ty.is_const();

                let ty = Item::from_ty(&ty, Some(cursor), None, context)
                            .expect("Unable to resolve constant type?");

                let mut value = None;

                if context.resolve_type(ty).is_integer_literal() {
                    // Try to parse a literal token value
                    cursor.visit(|c, _| {
                        if c.kind() == CXCursor_IntegerLiteral {
                            value =
                                parse_int_literal_tokens(&c,
                                                         context.translation_unit(),
                                                         0);
                        }
                        CXChildVisit_Continue
                    });
                }


                let var = Var::new(name, None, ty, value, is_const);
                println!("{:?}", var);
                Ok(ParseResult::New(var, Some(cursor)))

            }
            _ => {
                /* TODO */
                Err(ParseError::Continue)
            }
        }
    }
}

fn parse_int_literal_tokens(cursor: &clang::Cursor,
                            unit: &clang::TranslationUnit,
                            which: usize) -> Option<i64> {
    use clangll::CXToken_Literal;
    let tokens = match unit.tokens(cursor) {
        None => return None,
        Some(tokens) => tokens,
    };

    if tokens.len() <= which || tokens[which].kind != CXToken_Literal {
        return None;
    }

    let s = &tokens[which].spelling;
    // TODO: try to preserve hex literals?
    if s.starts_with("0x") {
        i64::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse().ok()
    }
}
