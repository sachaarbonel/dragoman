#[macro_use]
extern crate lazy_static;
use rustpython_parser::parser;

lazy_static! {
    static ref IDENTIFIER_MAP: std::collections::HashMap<String, &'static str> =
        vec![(r#"print"#.to_owned(), "println!")]
            .into_iter()
            .collect();
}

fn extract_type_string(value: &rustpython_parser::ast::StringGroup) -> ArgType {
    match value {
        rustpython_parser::ast::StringGroup::Constant { ref value } => ArgType::String {
            value: value.to_string(),
        },
        _ => unimplemented!(),
    }
}

// A certain type of expression.
#[derive(PartialEq)]
pub enum ExpressionType {
    FunctionCall {
        function_identifier: String,
        function_args: Vec<ArgType>,
    },
}

#[derive(PartialEq)]
pub enum ArgType {
    String { value: String },
}

impl std::fmt::Display for ArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &*self {
            ArgType::String { ref value } => write!(f, r#""{}""#, value),

            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for ExpressionType {
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
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),

            _ => unimplemented!(),
        }
    }
}

fn extract_call(
    function: &rustpython_parser::ast::Expression,
    args: &Vec<rustpython_parser::ast::Expression>,
) -> ExpressionType {
    let function = match function {
        rustpython_parser::ast::Located { location: _, node } => match node {
            rustpython_parser::ast::ExpressionType::Identifier { ref name } => name,
            _ => unimplemented!(),
        },
    };
    // println!("{}", function);
    let args = match args.as_slice() {
        [rustpython_parser::ast::Located { location: _, node }, ..] => match node {
            rustpython_parser::ast::ExpressionType::String { ref value } => {
                extract_type_string(value)
            }
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    ExpressionType::FunctionCall {
        function_identifier: function.to_owned(),
        function_args: vec![args],
    }
}

fn transpile(source: &str) -> String {
    let python_ast = parser::parse_expression(source).unwrap();
    let function_call = match python_ast {
        rustpython_parser::ast::Located { location: _, node } => match node {
            rustpython_parser::ast::ExpressionType::Call {
                //TODOs: handle other types https://docs.rs/rustpython-parser/0.1.2/src/rustpython_parser/ast.rs.html#179
                ref function,
                ref args,
                keywords: _,
            } => extract_call(function, args),
            _ => unimplemented!(),
        },
    };
    function_call.to_string()
}

fn main() {
    let python_source = r#"print("Hello world")"#;
    let func_call = transpile(python_source);
    println!("{:#?}", func_call);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_ser_hello_world() {
        let hello_world = r#"print("Hello world")"#;
        let hello_world_rs = transpile(hello_world);

        assert_eq!(hello_world_rs, "println!(\"Hello world\")");
    }
}
