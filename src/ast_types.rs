#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArgumentCollector {
    All, // collects all arguments to be passed on ...
    Rest, // collect "unclaimed" arguments
         //Positional, // collect all positional arguments
         //Keyword // collect keyword arguments
}

/// These are the basic building blocks of our casual lisp language.
/// You might notice that there's no lists in this lisp ... not sure
/// what to call it in that case ...
#[derive(Debug, Clone)]
pub enum Atom {
    Float(f32),
    String(String),
    Keyword(String),
    Symbol(String),
    Boolean(bool),
    Identifier(String),
    ArgumentCollector(ArgumentCollector),
}

/// Expression Type
#[derive(Debug, Clone)]
pub enum Expr {
    FunctionDefinition,
    VariableDefinition,
    PersistantStateDefinition,
    Constant(Atom),
    Application(Box<Expr>, Vec<Expr>),
    Definition(Box<Expr>, Vec<Expr>),
}
