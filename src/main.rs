#[macro_use]
extern crate lazy_static;
use rustpython_parser::ast;
use rustpython_parser::parser;

pub struct Python {}

lazy_static! {
    static ref IDENTIFIER_MAP: std::collections::HashMap<String, &'static str> =
        vec![(r#"print"#.to_owned(), "println!")]
            .into_iter()
            .collect();
}

fn extract_type_string(value: &ast::StringGroup) -> ExpressionType<Python> {
    match value {
        ast::StringGroup::Constant { ref value } => ExpressionType::<Python>::String {
            value: value.to_string(),
            phantom: std::marker::PhantomData,
        },
        _ => unimplemented!(),
    }
}

// A certain type of expression.
#[derive(PartialEq)]
pub enum Statement<T> {
    FunctionCall {
        function_identifier: String,
        function_args: Vec<ExpressionType<T>>,
    },

    List {
        elements: Vec<ExpressionType<T>>,
    },
}

#[derive(PartialEq)]
pub enum ExpressionType<T> {
    String {
        value: String,
        phantom: std::marker::PhantomData<T>,
    },
}

trait FormatTrait<T> {
    fn format(&self) -> String;
}

impl<T> FormatTrait<T> for Vec<T>
where
    T: std::string::ToString,
{
    fn format(&self) -> String {
        self.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

impl std::fmt::Display for ExpressionType<Python> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ExpressionType::<Python>::String {
                ref value,
                phantom: _,
            } => write!(f, r#""{}""#, value),

            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for Statement<Python> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            Statement::FunctionCall {
                ref function_identifier,
                function_args,
            } => write!(
                f,
                "{}({})",
                IDENTIFIER_MAP[&*function_identifier],
                function_args.format()
            ),
            Statement::List { elements } => write!(f, "vec![{}]", elements.format()),

            _ => unimplemented!(),
        }
    }
}

fn extract_call(function: &ast::Expression, args: &Vec<ast::Expression>) -> Statement<Python> {
    let function = match function {
        ast::Located { location: _, node } => match node {
            ast::ExpressionType::Identifier { ref name } => name,
            _ => unimplemented!(),
        },
    };
    let mut function_args = Vec::new();
    for arg in args {
        match arg {
            ast::Located {
                location: _,
                ref node,
            } => function_args.push(extract_expression_type(node)),
            _ => unimplemented!(),
        }
    }

    Statement::<Python>::FunctionCall {
        function_identifier: function.to_owned(),
        function_args: function_args,
    }
}

trait TranspilerTrait {
    fn transpile(source: &str) -> String;
}

fn extract_expression_type(arg: &ast::ExpressionType) -> ExpressionType<Python> {
    match arg {
        ast::ExpressionType::String { ref value } => extract_type_string(value),
        _ => unimplemented!(),
    }
}

fn extract_elements(elements: &Vec<ast::Expression>) -> Statement<Python> {
    let mut expression_types = Vec::new();
    for element in elements {
        match element {
            ast::Located {
                location: _,
                ref node,
            } => expression_types.push(extract_expression_type(node)),
        }
    }
    Statement::<Python>::List {
        elements: expression_types,
    }
}

fn extract_expression(expression_type: &ast::Located<ast::ExpressionType>) -> Statement<Python> {
    match expression_type {
        ast::Located { location: _, node } => match node {
            ast::ExpressionType::List { ref elements } => extract_elements(elements),
            ast::ExpressionType::Call {
                //TODOs: handle other types https://docs.rs/rustpython-parser/0.1.2/src/rustpython_parser/ast.rs.html#179
                ref function,
                ref args,
                keywords: _,
            } => extract_call(function, args),
            _ => unimplemented!(),
        },
    }
}

fn extract_statement(statement: &ast::Statement) -> Statement<Python> {
    match statement {
        ast::Statement {
            location: _,
            node: ast::StatementType::Expression { ref expression },
        } => extract_expression(expression),
        _ => unimplemented!(),
    }
}

impl TranspilerTrait for Python {
    fn transpile(source: &str) -> String {
        let mut statements = parser::parse_statement(source).unwrap();

        let mut result = Vec::new();
        while let Some(statement) = statements.pop() {
            let statement = extract_statement(&statement);
            result.push(statement.to_string());
        }
        result.join("\n")
    }
}

fn main() {
    let python_source = r#"["Apple", "Banana", "Dog"]"#;
    let func_call = parser::parse_statement(python_source);
    // let func_call = Python::transpile(python_source);
    println!("{:#?}", func_call);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_ser_print_hello() {
        let hello_world = r#"print("Hello world")"#;
        let hello_world_rs = Python::transpile(hello_world);

        assert_eq!(hello_world_rs, "println!(\"Hello world\")");
    }

    #[test]
    fn ast_ser_vec_string() {
        let hello_world = r#"["Apple", "Banana", "Dog"]"#;
        let hello_world_rs = Python::transpile(hello_world);

        assert_eq!(hello_world_rs, r#"vec!["Apple", "Banana", "Dog"]"#);
    }
}
