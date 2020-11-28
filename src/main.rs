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
pub enum Statement<T> {
    FunctionCall {
        function_identifier: String,
        function_args: Vec<ArgType<T>>,
    },

    List {
        elements: Vec<ArgType<T>>,
    },
}

#[derive(PartialEq)]
pub enum ArgType<T> {
    //rename to literal or something
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
                function_args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            Statement::List { elements } => write!(
                f,
                "vec![{}]",
                elements
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),

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
    // println!("{}", function);
    let args = match args.as_slice() {
        [ast::Located {
            location: _,
            ref node,
        }, ..] => extract_arg_type(node),
        _ => unimplemented!(),
    };
    Statement::<Python>::FunctionCall {
        function_identifier: function.to_owned(),
        function_args: vec![args],
    }
}

trait TranspilerTrait {
    fn transpile(source: &str) -> String;
}

fn extract_arg_type(arg: &ast::ExpressionType) -> ArgType<Python> {
    match arg {
        ast::ExpressionType::String { ref value } => extract_type_string(value),
        _ => unimplemented!(),
    }
}

fn extract_elements(elements: &Vec<ast::Expression>) -> Statement<Python> {
    let mut literals = Vec::new();
    for element in elements {
        match element {
            ast::Located {
                location: _,
                ref node,
            } => {
                let arg_type = extract_arg_type(node);
                literals.push(arg_type);
            }
        }
    }
    Statement::<Python>::List { elements: literals }
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
            let function_call = extract_statement(&statement);
            result.push(function_call.to_string());
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
