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

fn extract_type_string(value: &ast::StringGroup) -> ArgType<Python> {
    match value {
        ast::StringGroup::Constant { ref value } => ArgType::<Python>::String {
            value: value.to_string(),
            phantom: std::marker::PhantomData,
        },
        _ => unimplemented!(),
    }
}

// A certain type of expression.
#[derive(PartialEq)]
pub enum ExpressionType<T> {
    FunctionCall {
        function_identifier: String,
        function_args: Vec<ArgType<T>>,
    },
}

#[derive(PartialEq)]
pub enum ArgType<T> {
    String {
        value: String,
        phantom: std::marker::PhantomData<T>,
    },
}

impl std::fmt::Display for ArgType<Python> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ArgType::<Python>::String {
                ref value,
                phantom: _,
            } => write!(f, r#""{}""#, value),

            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for ExpressionType<Python> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ExpressionType::FunctionCall {
                ref function_identifier,
                function_args,
            } => write!(
                f,
                "{}({})",
                IDENTIFIER_MAP[&*function_identifier],
                function_args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),

            _ => unimplemented!(),
        }
    }
}

fn extract_call(function: &ast::Expression, args: &Vec<ast::Expression>) -> ExpressionType<Python> {
    let function = match function {
        ast::Located { location: _, node } => match node {
            ast::ExpressionType::Identifier { ref name } => name,
            _ => unimplemented!(),
        },
    };
    // println!("{}", function);
    let args = match args.as_slice() {
        [ast::Located { location: _, node }, ..] => match node {
            ast::ExpressionType::String { ref value } => extract_type_string(value),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    ExpressionType::<Python>::FunctionCall {
        function_identifier: function.to_owned(),
        function_args: vec![args],
    }
}

trait TranspilerTrait {
    fn transpile(source: &str) -> String;
}

fn extract_expression_type(
    expression_type: &ast::Located<ast::ExpressionType>,
) -> ExpressionType<Python> {
    match expression_type {
        ast::Located { location: _, node } => match node {
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

impl TranspilerTrait for Python {
    fn transpile(source: &str) -> String {
        let statement = &parser::parse_statement(source).unwrap()[0];
        let function_call = match statement {
            ast::Statement {
                location: _,
                node: ast::StatementType::Expression { ref expression },
            } => extract_expression_type(expression),
            _ => unimplemented!(),
        };
        // let function_call = extract_expression_type(python_ast);
        function_call.to_string()    }
}

fn main() {
    let python_source = r#"print("Hello world")"#;
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
}
