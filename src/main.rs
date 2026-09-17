use calico_parser::parser::Parser;

fn main() {
    let src = r#"
package me.blinx;
import java.util.List;
import java.util.Map;
import static java.lang.Math.PI;

public class Main {
    public static final volatile transient synchronized native strictfp int x = 42;
}
"#;

    println!("{}", src);

    let mut parser = Parser::new(src);
    match parser.parse() {
        Ok(unit) => println!("{:#?}", unit),
        Err(e) => eprintln!("parse error: {:?}", e),
    }
}
