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

#[derive(Debug, PartialEq)]
pub struct CompilationUnit {
    pub package: Option<PackageDecl>,
    pub imports: Vec<ImportDecl>,
    pub types: Vec<TypeDecl>,
}

#[derive(Debug, PartialEq)]
pub struct PackageDecl {
    pub name: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct ImportDecl {
    pub static_: bool,
    pub path: Vec<String>,
    pub wildcard: bool,
}

#[derive(Debug, PartialEq)]
pub enum TypeDecl {
    Class(ClassDecl),
    Interface(InterfaceDecl),
    Enum(EnumDecl),
    Annotation(AnnotationDecl),
}

#[derive(Debug, PartialEq)]
pub struct ClassDecl {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub superclass: Option<TypeExpr>,
    pub interfaces: Vec<TypeExpr>,
    pub body: Vec<ClassMember>,
}

#[derive(Debug, PartialEq)]
pub struct InterfaceDecl {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub extends: Vec<TypeExpr>,
    pub body: Vec<InterfaceMember>,
}

#[derive(Debug, PartialEq)]
pub struct EnumDecl {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub interfaces: Vec<TypeExpr>,
    pub constants: Vec<EnumConstant>,
    pub body: Vec<ClassMember>,
}

#[derive(Debug, PartialEq)]
pub struct AnnotationDecl {
    pub modifiers: Vec<Modifier>,
    pub name: String,
    pub body: Vec<AnnotationMember>,
}

#[derive(Debug, PartialEq)]
pub struct EnumConstant {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub args: Option<Vec<Expr>>,
    pub body: Option<Vec<ClassMember>>,
}

#[derive(Debug, PartialEq)]
pub enum ClassMember {
    Field(FieldDecl),
    Method(MethodDecl),
    Constructor(ConstructorDecl),
    StaticInit(Vec<Stmt>),
    InstanceInit(Vec<Stmt>),
    InnerType(TypeDecl),
}

#[derive(Debug, PartialEq)]
pub enum InterfaceMember {
    Method(MethodDecl),
    Constant(FieldDecl),
    InnerType(TypeDecl),
    Default(MethodDecl),
}

#[derive(Debug, PartialEq)]
pub enum AnnotationMember {
    Element {
        name: String,
        ty: TypeExpr,
        default: Option<Expr>,
    },
    Constant(FieldDecl),
}

#[derive(Debug, PartialEq)]
pub struct FieldDecl {
    pub modifiers: Vec<Modifier>,
    pub ty: TypeExpr,
    pub declarators: Vec<VarDeclarator>,
}

#[derive(Debug, PartialEq)]
pub struct VarDeclarator {
    pub name: String,
    pub dims: usize,
    pub init: Option<VarInit>,
}

#[derive(Debug, PartialEq)]
pub enum VarInit {
    Expr(Expr),
    Array(ArrayInit),
}

#[derive(Debug, PartialEq)]
pub struct ArrayInit {
    pub elements: Vec<VarInit>,
}

#[derive(Debug, PartialEq)]
pub struct MethodDecl {
    pub modifiers: Vec<Modifier>,
    pub type_params: Vec<TypeParam>,
    pub return_ty: TypeExpr,
    pub name: String,
    pub params: Vec<Param>,
    pub throws: Vec<TypeExpr>,
    pub body: Option<Vec<Stmt>>,
}

#[derive(Debug, PartialEq)]
pub struct ConstructorDecl {
    pub modifiers: Vec<Modifier>,
    pub type_params: Vec<TypeParam>,
    pub name: String,
    pub params: Vec<Param>,
    pub throws: Vec<TypeExpr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
pub struct Param {
    pub modifiers: Vec<Modifier>,
    pub ty: TypeExpr,
    pub varargs: bool,
    pub name: String,
    pub dims: usize,
}

#[derive(Debug, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TypeExpr>,
}

#[derive(Debug, PartialEq)]
pub enum TypeExpr {
    Primitive(PrimitiveType),
    Void,
    Named {
        name: Vec<String>,
        type_args: Vec<TypeArg>,
    },
    Array(Box<TypeExpr>, usize),
}

#[derive(Debug, PartialEq)]
pub enum PrimitiveType {
    Boolean, Byte, Short, Int, Long, Char, Float, Double
}

#[derive(Debug, PartialEq)]
pub enum TypeArg {
    Type(TypeExpr),
    Wildcard(Option<WildcardBound>)
}

#[derive(Debug, PartialEq)]
pub enum WildcardBound {
    Extends(TypeExpr),
    Super(TypeExpr)
}

#[derive(Debug, PartialEq)]
pub enum Modifier {
    Public, Protected, Private,
    Static, Final, Abstract, Native,
    Synchronized, Transient, Volatile, Strictfp,
    Default, Annotation(Annotation)
}

#[derive(Debug, PartialEq)]
pub struct Annotation {
    pub name: Vec<String>,
    pub args: AnnotationArgs,
}

#[derive(Debug, PartialEq)]
pub enum AnnotationArgs {
    None,
    Single(Expr),
    Named(Vec<(String, Expr)>)
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Empty,
    Block(Vec<Stmt>),
    LocalVar {
        modifiers: Vec<Modifier>,
        ty: TypeExpr,
        declarators: Vec<VarDeclarator>
    },
    Expr(Expr),
    If {
        cond: Box<Expr>,
        then: Box<Stmt>,
        else_: Option<Box<Stmt>>
    },
    While {
        cond: Box<Expr>,
        body: Box<Stmt>
    },
    DoWhile {
        body: Box<Stmt>,
        cond: Box<Expr>,
    },
    For(ForStmt),
    Switch {
        expr: Box<Expr>,
        groups: Vec<SwitchGroup>,
    },
    Return(Option<Box<Expr>>),
    Throw(Box<Expr>),
    Break(Option<String>),
    Continue(Option<String>),
    Labeled {
        label: String,
        stmt: Box<Stmt>,
    },
    Try(TryStmt),
    Synchronized {
        lock: Box<Expr>,
        body: Vec<Stmt>,
    },
    Assert {
        cond: Box<Expr>,
        msg: Option<Box<Expr>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ForStmt {
    Basic {
        init: Vec<Stmt>,
        cond: Option<Box<Expr>>,
        update: Vec<Expr>,
        body: Box<Stmt>,
    },
    Enhanced {
        modifiers: Vec<Modifier>,
        ty: TypeExpr,
        var: String,
        iter: Box<Expr>,
        body: Box<Stmt>,
    },
}

#[derive(Debug, PartialEq)]
pub struct SwitchGroup {
    pub labels: Vec<SwitchLabel>,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
pub enum SwitchLabel {
    Case(Expr),
    Default,
}

#[derive(Debug, PartialEq)]
pub struct TryStmt {
    pub resources: Vec<TryResource>,
    pub body: Vec<Stmt>,
    pub catches: Vec<CatchClause>,
    pub finally: Option<Vec<Stmt>>,
}

#[derive(Debug, PartialEq)]
pub struct TryResource {
    pub modifiers: Vec<Modifier>,
    pub ty: TypeExpr,
    pub name: String,
    pub init: Expr,
}

#[derive(Debug, PartialEq)]
pub struct CatchClause {
    pub modifiers: Vec<Modifier>,
    pub types: Vec<TypeExpr>,
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    IntLiteral(i64),
    LongLiteral(i64),
    FloatLiteral(f64),
    DoubleLiteral(f64),
    BoolLiteral(bool),
    CharLiteral(char),
    StringLiteral(String),
    Null,
    Ident(String),
    This,
    Super,
    ClassLiteral(TypeExpr),
    MethodRef {
        receiver: MethodRefReceiver,
        method: String,
    },
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    PostfixOp {
        op: PostfixOp,
        expr: Box<Expr>,
    },
    Ternary {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    Assign {
        op: AssignOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Instanceof {
        expr: Box<Expr>,
        ty: TypeExpr,
    },
    Cast {
        ty: TypeExpr,
        expr: Box<Expr>,
    },
    FieldAccess {
        receiver: Box<Expr>,
        field: String,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    MethodCall {
        receiver: Option<Box<Expr>>,
        type_args: Vec<TypeArg>,
        name: String,
        args: Vec<Expr>,
    },
    NewObject {
        type_args: Vec<TypeArg>,
        ty: TypeExpr,
        args: Vec<Expr>,
        body: Option<Vec<ClassMember>>,
    },
    NewArray {
        ty: TypeExpr,
        dims: Vec<Option<Expr>>,
        init: Option<ArrayInit>,
    },
    Lambda {
        params: LambdaParams,
        body: LambdaBody,
    },
}

#[derive(Debug, PartialEq)]
pub enum MethodRefReceiver {
    Type(TypeExpr),
    Expr(Box<Expr>),
    Super,
    New,
}

#[derive(Debug, PartialEq)]
pub enum LambdaParams {
    Inferred(Vec<String>),
    Typed(Vec<Param>),
    Single(String),
}

#[derive(Debug, PartialEq)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    And, Or, Xor,
    Shl, Shr, UShr,
    AndAnd, OrOr,
    Eq, Ne, Lt, Gt, Le, Ge,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Plus, Neg, Not, BitNot, PreInc, PreDec,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PostfixOp {
    Inc, Dec,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignOp {
    Assign,
    Add, Sub, Mul, Div, Rem,
    And, Or, Xor,
    Shl, Shr, UShr,
}