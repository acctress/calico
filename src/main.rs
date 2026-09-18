use calico_parser::parser::Parser;

fn main() {
    let src = r#"
package me.blinx;

public class Main {
    public void errors() {
        try {
            throw new RuntimeException("oops");
        } catch (RuntimeException e) {
            System.out.println(e);
        } finally {
            System.out.println("done");
        }

        try (BufferedReader r = new BufferedReader(new FileReader("f.txt"))) {
            String line = r.readLine();
        } catch (IOException | RuntimeException e) {
            e.printStackTrace();
        }
    }
}
"#;

    let mut parser = Parser::new(src);
    match parser.parse() {
        Ok(unit) => println!("{:#?}", unit),
        Err(e)   => eprintln!("parse error: {:?}", e),
    }
}