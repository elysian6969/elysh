use elysh_syntax::Vars;

fn main() {
    let command_line = "KEY";
    println!("{:?}", command_line);
    let mut vars = Vars::new(command_line);

    println!("{:?}", vars.next());

    let command_line = "KEY=";
    println!("{:?}", command_line);
    let mut vars = Vars::new(command_line);

    println!("{:?}", vars.next());
    println!("{:?}", vars.next());

    let command_line = "KEY=val";
    println!("{:?}", command_line);
    let mut vars = Vars::new(command_line);

    println!("{:?}", vars.next());
    println!("{:?}", vars.next());

    let command_line = "KEY=\"val";
    println!("{:?}", command_line);
    let mut vars = Vars::new(command_line);

    println!("{:?}", vars.next());
    println!("{:?}", vars.next());

    let command_line = "KEY=\"val\"";
    println!("{:?}", command_line);
    let mut vars = Vars::new(command_line);

    println!("{:?}", vars.next());
    println!("{:?}", vars.next());

    let command_line = "KEY=\"val\" FOO=bar";
    println!("{:?}", command_line);
    let mut vars = Vars::new(command_line);

    println!("{:?}", vars.next());
    println!("{:?}", vars.next());
    println!("{:?}", vars.next());
    println!("{:?}", vars.next());
}
