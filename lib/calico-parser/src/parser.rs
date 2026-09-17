/*
 * Copyright (c) 2026 acctress.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

use crate::lexer::{Token, Lexer};
use crate::ast::*;

#[derive(Debug)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub enum ParseErr {
    InvalidModifier(Token, Span),
    Unexpected {
        expected: Token,
        found: Token,
        span: Span,
        msg: &'static str,
    },
    UnexpectedToken(Token, Span),
    UnexpectedEof,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
    next: Option<Token>
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut p = Self {
            lexer: Lexer::new(source),
            current: None,
            next: None,
        };

        p.consume();
        p.consume();
        p
    }

    fn consume(&mut self) -> Option<Token> {
        let p = self.current.take();
        self.current = self.next.take();
        self.next = self.lexer.next();
        p
    }

    fn expect(&mut self, tk: Token, msg: &'static str) -> Result<Token, ParseErr> {
        match &self.current {
            Some(c) if *c == tk => Ok(self.consume().unwrap()),
            Some(c) => Err(ParseErr::Unexpected {
                expected: tk,
                found: c.clone(),
                span: Span { line: 0, col: 0 },
                msg,
            }),
            None => Err(ParseErr::UnexpectedEof),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn peek2(&self) -> Option<&Token> {
        self.next.as_ref()
    }

    fn get_identifier_value(&mut self) -> Result<String, ParseErr> {
        match self.consume() {
            Some(Token::Identifier(s)) => Ok(s),
            Some(t) => Err(ParseErr::UnexpectedToken(t, Span { line: 0, col: 0 })),
            None => Err(ParseErr::UnexpectedEof),
        }
    }

    pub fn parse(&mut self) -> Result<CompilationUnit, ParseErr> {
        // compilation_unit <- package_decl? import_decl* type_decl*
        let package = if self.peek() == Some(&Token::Package) {
            Some(self.parse_package_decl()?)
        } else {
            None
        };

        let mut imports = vec![];
        while self.peek() == Some(&Token::Import) {
            imports.push(self.parse_import_decl()?);
        }

        let mut types = vec![];
        while self.current.is_some() {
            let modifiers = self.parse_modifiers()?;
            types.push(self.parse_type_decl(modifiers)?);
        }

        Ok(CompilationUnit { package, imports, types } )
    }

    fn parse_package_decl(&mut self) -> Result<PackageDecl, ParseErr> {
        // package_decl <- 'package' qualified_name ';'
        self.consume();

        let name = self.parse_qualified_name()?;
        self.expect(Token::Semicolon, "expected ';' after package name.")?;
        Ok(PackageDecl { name })
    }

    fn parse_import_decl(&mut self) -> Result<ImportDecl, ParseErr> {
        // import_decl <- 'import' 'static'? qualified_name ('.' '*')? ';'
        self.consume();

        let static_ = if self.peek() == Some(&Token::Static) {
            self.consume();
            true
        } else {
            false
        };

        let path = self.parse_qualified_name()?;
        let wildcard = if self.peek() == Some(&Token::Dot) && self.peek2() == Some(&Token::Star) {
            self.consume(); self.consume();
            true
        } else {
            false
        };

        self.expect(Token::Semicolon, "expected ';' after import")?;
        Ok(ImportDecl { static_, path, wildcard })
    }

    fn parse_type_decl(&mut self, modifiers: Vec<Modifier>) -> Result<TypeDecl, ParseErr> {
        // type_decl <- class_decl | interface_decl | enum_decl | annotation_decl
        match self.peek() {
            Some(Token::Class)     => Ok(TypeDecl::Class(self.parse_class_decl(modifiers)?)),
            Some(Token::Interface) => Ok(TypeDecl::Interface(self.parse_interface_decl(modifiers)?)),
            Some(Token::Enum)      => Ok(TypeDecl::Enum(self.parse_enum_decl(modifiers)?)),
            Some(Token::At) if self.peek2() == Some(&Token::Interface) => {
                Ok(TypeDecl::Annotation(self.parse_annotation_decl(modifiers)?))
            }
            Some(t) => Err(ParseErr::UnexpectedToken(t.clone(), Span { line: 0, col: 0 })),
            None    => Err(ParseErr::UnexpectedEof),
        }
    }

    fn parse_class_decl(&mut self, modifiers: Vec<Modifier>) -> Result<ClassDecl, ParseErr> {
        // class_decl <- 'class' ident type_params? ('extends' type)? ('implements' type_list)? class_body
        self.consume();
        let name = self.get_identifier_value()?;

        let type_params = if self.peek() == Some(&Token::Lt) {
            self.parse_type_params()?
        } else {
            vec![]
        };

        let superclass = if self.peek() == Some(&Token::Extends) {
            self.consume();
            Some(self.parse_type()?)
        } else {
            None
        };

        let interfaces = if self.peek() == Some(&Token::Implements) {
            self.consume();
            let mut types = vec![];
            loop {
                types.push(self.parse_type()?);
                if self.peek() == Some(&Token::Comma) {
                    self.consume();
                } else {
                    break;
                }
            }

            types
        } else {
            vec![]
        };

        let body = self.parse_class_body()?;
        Ok(ClassDecl {
            modifiers,
            name,
            type_params,
            superclass,
            interfaces,
            body
        })
    }

    fn parse_interface_decl(&mut self, modifiers: Vec<Modifier>) -> Result<InterfaceDecl, ParseErr> {
        // interface_decl <- 'interface' ident type_params? ('extends' type_list)? interface_body
        todo!()
    }

    fn parse_enum_decl(&mut self, modifiers: Vec<Modifier>) -> Result<EnumDecl, ParseErr> {
        // enum_decl <- 'enum' ident ('implements' type_list)? '{' enum_constants (',' ';' class_body_decl*)? '}'
        todo!()
    }

    fn parse_annotation_decl(&mut self, modifiers: Vec<Modifier>) -> Result<AnnotationDecl, ParseErr> {
        // annotation_decl <- '@' 'interface' ident '{' annotation_member* '}'
        todo!()
    }

    fn parse_class_body(&mut self) -> Result<Vec<ClassMember>, ParseErr> {
        // class_body <- '{' class_member* '}'
        self.expect(Token::LBrace, "expected '{'")?;

        let mut members = vec![];
        while self.peek() != Some(&Token::RBrace) {
            if self.current.is_none() { return Err(ParseErr::UnexpectedEof); }
            members.push(self.parse_class_member()?);
        }

        self.expect(Token::RBrace, "expected '}'")?;
        Ok(members)
    }

    fn parse_class_member(&mut self) -> Result<ClassMember, ParseErr> {
        // class_member <- modifiers (field_decl | method_decl | constructor_decl | static_init | instance_init | type_decl)
        if self.peek() == Some(&Token::LBrace) {
            let body = self.parse_block()?;
            return Ok(ClassMember::InstanceInit(body));
        }

        let modifiers = self.parse_modifiers()?;
        if modifiers.iter().any(|m| *m == Modifier::Static) && self.peek() == Some(&Token::LBrace) {
            let body = self.parse_block()?;
            return Ok(ClassMember::StaticInit(body));
        }

        match self.peek() {
            Some(Token::Class) | Some(Token::Interface) | Some(Token::Enum) => {
                return Ok(ClassMember::InnerType(self.parse_type_decl(modifiers)?));
            }

            Some(Token::At) if self.peek2() == Some(&Token::Interface) => {
                return Ok(ClassMember::InnerType(self.parse_type_decl(modifiers)?));
            }

            _ => {}
        }

        if let Some(Token::Identifier(_)) = self.peek() {
            if self.peek2() == Some(&Token::LParen) {
                let name = self.get_identifier_value()?;
                return Ok(ClassMember::Constructor(self.parse_constructor_decl(modifiers, name)?));
            }
        }

        let ty = self.parse_type()?;
        let name = self.get_identifier_value()?;

        if self.peek() == Some(&Token::LParen) {
            Ok(ClassMember::Method(self.parse_method_decl(modifiers, ty, name)?))
        } else {
            Ok(ClassMember::Field(self.parse_field_decl(modifiers, ty, name)?))
        }
    }

    fn parse_field_decl(&mut self, modifiers: Vec<Modifier>, ty: TypeExpr, name: String) -> Result<FieldDecl, ParseErr> {
        // field_decl <- type var_declarator (',' var_declarator)* ';'
        let mut declarators = vec![self.parse_var_declarator_wn(name)?];
        while self.peek() == Some(&Token::Comma) {
            self.consume();
            declarators.push(self.parse_var_declarator()?);
        }

        self.expect(Token::Semicolon, "expected ';' after field")?;
        Ok(FieldDecl { modifiers, ty, declarators })
    }

    fn parse_var_declarator_wn(&mut self, name: String) -> Result<VarDeclarator, ParseErr> {
        let mut pairs = 0;
        while self.peek() == Some(&Token::LBracket) && self.peek2() == Some(&Token::RBracket) {
            self.consume(); self.consume();
            pairs += 1;
        }

        let init = if self.peek() == Some(&Token::Assign) {
            self.consume();
            Some(self.parse_var_init()?)
        } else {
            None
        };

        Ok(VarDeclarator { name, dims: pairs, init })
    }

    fn parse_var_declarator(&mut self) -> Result<VarDeclarator, ParseErr> {
        let name = self.get_identifier_value()?;
        self.parse_var_declarator_wn(name)
    }

    fn parse_var_init(&mut self) -> Result<VarInit, ParseErr> {
        if self.peek() == Some(&Token::LBrace) {
            Ok(VarInit::Array(self.parse_array_init()?))
        } else {
            Ok(VarInit::Expr(self.parse_expr()?))
        }
    }

    fn parse_method_decl(&mut self, modifiers: Vec<Modifier>, return_ty: TypeExpr, name: String) -> Result<MethodDecl, ParseErr> {
        // method_decl <- type_params? type ident '(' params? ')' ('[' ']')* throws? (block | ';')
        todo!()
    }

    fn parse_constructor_decl(&mut self, modifiers: Vec<Modifier>, name: String) -> Result<ConstructorDecl, ParseErr> {
        // constructor_decl <- type_params? ident '(' params? ')' throws? block
        todo!()
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseErr> {
        // params <- param (',' param)* (',' '...' param)?
        todo!()
    }

    fn parse_param(&mut self) -> Result<Param, ParseErr> {
        // param <- modifiers? type '...'? ident ('[' ']')*
        todo!()
    }

    fn parse_modifiers(&mut self) -> Result<Vec<Modifier>, ParseErr> {
        // modifiers <- (modifier | annotation)*
        // modifier  <- 'public' | 'protected' | 'private' | 'static' | 'final'
        //            | 'abstract' | 'native' | 'synchronized' | 'transient'
        //            | 'volatile' | 'strictfp' | 'default'

        let mut modifiers = vec![];
        loop {
            match self.peek() {
                Some(Token::Public) => { self.consume(); modifiers.push(Modifier::Public); }
                Some(Token::Protected) => { self.consume(); modifiers.push(Modifier::Protected); }
                Some(Token::Private) => { self.consume(); modifiers.push(Modifier::Private); }
                Some(Token::Static) => { self.consume(); modifiers.push(Modifier::Static); }
                Some(Token::Final) => { self.consume(); modifiers.push(Modifier::Final); }
                Some(Token::Abstract) => { self.consume(); modifiers.push(Modifier::Abstract); }
                Some(Token::Native) => { self.consume(); modifiers.push(Modifier::Native); }
                Some(Token::Synchronized) => { self.consume(); modifiers.push(Modifier::Synchronized); }
                Some(Token::Transient) => { self.consume(); modifiers.push(Modifier::Transient); }
                Some(Token::Volatile) => { self.consume(); modifiers.push(Modifier::Volatile); }
                Some(Token::Strictfp) => { self.consume(); modifiers.push(Modifier::Strictfp); }
                Some(Token::Default) => { self.consume(); modifiers.push(Modifier::Default); }
                Some(Token::At) => { modifiers.push(Modifier::Annotation(self.parse_annotation()?)); }
                _ => break,
            }
        }

        Ok(modifiers)
    }

    fn parse_annotation(&mut self) -> Result<Annotation, ParseErr> {
        // annotation <- '@' qualified_name ('(' annotation_args? ')')?
        todo!()
    }

    fn parse_type(&mut self) -> Result<TypeExpr, ParseErr> {
        // type <- primitive_type ('[' ']')* | ref_type ('[' ']')*
        let ty = match self.peek() {
            Some(Token::Identifier(_)) => self.parse_ref_type()?,
            Some(Token::Boolean) => { self.consume(); TypeExpr::Primitive(PrimitiveType::Boolean) }
            Some(Token::Byte)    => { self.consume(); TypeExpr::Primitive(PrimitiveType::Byte) }
            Some(Token::Short)   => { self.consume(); TypeExpr::Primitive(PrimitiveType::Short) }
            Some(Token::Int)     => { self.consume(); TypeExpr::Primitive(PrimitiveType::Int) }
            Some(Token::Long)    => { self.consume(); TypeExpr::Primitive(PrimitiveType::Long) }
            Some(Token::Char)    => { self.consume(); TypeExpr::Primitive(PrimitiveType::Char) }
            Some(Token::Float)   => { self.consume(); TypeExpr::Primitive(PrimitiveType::Float) }
            Some(Token::Double)  => { self.consume(); TypeExpr::Primitive(PrimitiveType::Double) }
            Some(t) => return Err(ParseErr::UnexpectedToken(t.clone(), Span { line: 0, col: 0 })),
            None    => return Err(ParseErr::UnexpectedEof),
        };

        let mut pairs = 0;
        while self.peek() == Some(&Token::LBracket) && self.peek2() == Some(&Token::RBracket) {
            self.consume(); self.consume();
            pairs += 1;
        }

        if pairs > 0 {
            Ok(TypeExpr::Array(Box::new(ty), pairs))
        } else {
            Ok(ty)
        }
    }

    fn parse_ref_type(&mut self) -> Result<TypeExpr, ParseErr> {
        // ref_type <- qualified_name type_args?
        let name = self.parse_qualified_name()?;
        let type_args = if self.peek() == Some(&Token::Lt) {
            self.consume();
            let args = self.parse_type_args()?;
            self.expect(Token::Gt, "expected '>' after type")?;

            args
        } else {
            vec![]
        };

        Ok(TypeExpr::Named {
            name,
            type_args
        })
    }

    fn parse_type_args(&mut self) -> Result<Vec<TypeArg>, ParseErr> {
        // type_args <- '<' type_arg (',' type_arg)* '>'
        // note: '>' may need to be reconstructed from '>>' token
        Ok(vec![])
    }

    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, ParseErr> {
        // type_params <- '<' type_param (',' type_param)* '>'
        Ok(vec![])
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseErr> {
        // block <- '{' stmt* '}'
        todo!()
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // stmt <- block | if_stmt | for_stmt | while_stmt | do_stmt | try_stmt
        //       | switch_stmt | synchronized_stmt | return_stmt | throw_stmt
        //       | break_stmt | continue_stmt | assert_stmt | labeled_stmt
        //       | local_var_decl | expr_stmt | ';'
        todo!()
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // if_stmt <- 'if' '(' expr ')' stmt ('else' stmt)?
        todo!()
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // for_stmt <- 'for' '(' (enhanced_for | basic_for) ')'  stmt
        // enhanced  <- modifiers? type ident ':' expr
        // basic     <- for_init? ';' expr? ';' for_update?
        todo!()
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // while_stmt <- 'while' '(' expr ')' stmt
        todo!()
    }

    fn parse_do_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // do_stmt <- 'do' stmt 'while' '(' expr ')' ';'
        todo!()
    }

    fn parse_try_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // try_stmt <- 'try' resource_spec? block catch_clause* finally_clause?
        // resource_spec <- '(' try_resource (';' try_resource)* ';'? ')'
        todo!()
    }

    fn parse_switch_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // switch_stmt <- 'switch' '(' expr ')' '{' switch_group* '}'
        // switch_group <- switch_label+ stmt+
        todo!()
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // return_stmt <- 'return' expr? ';'
        todo!()
    }

    fn parse_throw_stmt(&mut self) -> Result<Stmt, ParseErr> {
        // throw_stmt <- 'throw' expr ';'
        todo!()
    }

    fn parse_local_var_decl(&mut self) -> Result<Stmt, ParseErr> {
        // local_var_decl <- modifiers? type var_declarator (',' var_declarator)* ';'
        todo!()
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseErr> {
        // entry point — assignment is lowest precedence
        // expr <- assignment
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseErr> {
        let lhs = self.parse_ternary()?;
        let op = match self.peek() {
            Some(Token::Assign)      => AssignOp::Assign,
            Some(Token::PlusAssign)  => AssignOp::Add,
            Some(Token::MinusAssign) => AssignOp::Sub,
            Some(Token::StarAssign)  => AssignOp::Mul,
            Some(Token::SlashAssign) => AssignOp::Div,
            Some(Token::PercentAssign) => AssignOp::Rem,
            Some(Token::AmpAssign)   => AssignOp::And,
            Some(Token::PipeAssign)  => AssignOp::Or,
            Some(Token::CaretAssign) => AssignOp::Xor,
            Some(Token::ShlAssign)   => AssignOp::Shl,
            Some(Token::ShrAssign)   => AssignOp::Shr,
            Some(Token::UShrAssign)  => AssignOp::UShr,
            _ => return Ok(lhs),
        };

        self.consume();
        let rhs = self.parse_assignment()?;
        Ok(Expr::Assign { op, lhs: Box::new(lhs), rhs: Box::new(rhs) })
    }
    fn parse_ternary(&mut self) -> Result<Expr, ParseErr>    {
        let c = self.parse_or()?;
        if self.peek() == Some(&Token::Question) {
            self.consume();

            let then = self.parse_assignment()?;
            self.expect(Token::Colon, "expected ':' in ternary")?;
            let els = self.parse_assignment()?;
            return Ok(Expr::Ternary {
                cond: Box::new(c),
                then: Box::new(then),
                else_: Box::new(els)
            })
        }

        Ok(c)
    }
    fn parse_or(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_and()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::PipePipe => BinOp::OrOr,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_and()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_and(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_bit_or()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::AmpAmp => BinOp::AndAnd,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_bit_or()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_bit_or(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_bit_xor()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::Pipe => BinOp::Or,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_bit_xor()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_bit_xor(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_bit_and()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::Caret => BinOp::Xor,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_bit_and()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_bit_and(&mut self) -> Result<Expr, ParseErr>    {
        let mut lhs = self.parse_equality()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::Amp => BinOp::And,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_equality()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_equality(&mut self) -> Result<Expr, ParseErr>   {
        let mut lhs = self.parse_relational()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::EqEq => BinOp::Eq,
                Token::BangEq => BinOp::Ne,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_relational()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_relational(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_shift()?;
        while let Some(t) = &self.peek() {
            if matches!(t, Token::Instanceof) {
                self.consume();

                let ty = self.parse_type()?;
                lhs = Expr::Instanceof { expr: Box::new(lhs), ty };
                continue;
            }

            let op = match t {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::Le,
                Token::GtEq => BinOp::Ge,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_shift()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseErr>      {
        let mut lhs = self.parse_additive()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::Shl => BinOp::Shl,
                Token::Shr => BinOp::Shr,
                Token::UShr => BinOp::UShr,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_additive()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }
    fn parse_additive(&mut self) -> Result<Expr, ParseErr>   {
        let mut lhs = self.parse_multiplicative()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseErr> {
        let mut lhs = self.parse_unary()?;
        while let Some(t) = &self.peek() {
            let op = match t {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Rem,
                _ => break,
            };

            self.consume();
            let rhs = self.parse_unary()?;
            lhs = Expr::BinOp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseErr>      { self.parse_primary() }

    fn parse_cast(&mut self) -> Result<Expr, ParseErr> {
        // cast_expr <- '(' primitive_type ')' unary
        //            | '(' ref_type ')' unary  -- ambiguous with paren expr; use peek2
        todo!()
    }

    fn parse_postfix(&mut self, expr: Expr) -> Result<Expr, ParseErr> {
        // postfix <- primary (('.' | '::') ... | '[' expr ']' | '++' | '--')*
        todo!()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseErr> {
        // primary <- literal | 'this' | 'super' | ident | '(' expr ')'
        //          | new_expr | method_call | class_literal | lambda_expr
        match self.consume() {
            Some(Token::StringLiteral(s))  => Ok(Expr::StringLiteral(s)),
            Some(Token::IntegerLiteral(n)) => Ok(Expr::IntLiteral(n)),
            Some(Token::FloatingPointLiteral(f)) => Ok(Expr::DoubleLiteral(f)),
            Some(Token::BooleanLiteral(b)) => Ok(Expr::BoolLiteral(b)),
            Some(Token::CharacterLiteral(c)) => Ok(Expr::CharLiteral(c)),
            Some(Token::NullLiteral)       => Ok(Expr::Null),
            Some(Token::This)              => Ok(Expr::This),
            Some(Token::Identifier(s))     => Ok(Expr::Ident(s)),
            Some(t) => Err(ParseErr::UnexpectedToken(t, Span { line: 0, col: 0 })),
            None    => Err(ParseErr::UnexpectedEof),
        }
    }

    fn parse_new_expr(&mut self) -> Result<Expr, ParseErr> {
        // new_expr <- 'new' type_args? (class_creator | array_creator)
        // class_creator <- type '(' args? ')' class_body?
        // array_creator <- type ('[' expr ']')+ ('[' ']')* | type ('[' ']')+ array_init
        todo!()
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseErr> {
        // lambda_expr <- lambda_params '->' (expr | block)
        // lambda_params <- ident | '(' ')' | '(' ident (',' ident)* ')' | '(' typed_params ')'
        todo!()
    }

    fn parse_method_ref(&mut self, receiver: MethodRefReceiver) -> Result<Expr, ParseErr> {
        // method_ref <- (type | expr | 'super') '::' type_args? (ident | 'new')
        todo!()
    }

    fn parse_qualified_name(&mut self) -> Result<Vec<String>, ParseErr> {
        let mut p = vec![self.get_identifier_value()?];
        while self.peek() == Some(&Token::Dot) {
            self.consume();
            p.push(self.get_identifier_value()?);
        }

        Ok(p)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseErr> {
        // args <- '(' (expr (',' expr)*)? ')'
        todo!()
    }

    fn parse_array_init(&mut self) -> Result<ArrayInit, ParseErr> {
        // array_init <- '{' (var_init (',' var_init)* ','?)? '}'
        todo!()
    }
}