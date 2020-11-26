#[macro_use]
extern crate quote;
extern crate syn;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_ser() {
       let source = r#"
        fn main() {
            let string = "line one
            line two";
        }
    "#;

    let syntax = syn::parse_file(source).unwrap();

        assert!(larger.can_hold(&smaller));
    }
}