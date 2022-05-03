use elysh_syntax::Args;

fn main() {
    let command_line = "\"hi\"word";
    println!("{:?}", command_line);
    let mut args = Args::new(command_line);

    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());

    let command_line = "word\"hi\"word";
    println!("{:?}", command_line);
    let mut args = Args::new(command_line);

    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());

    let command_line = "word \"hi\" word";
    println!("{:?}", command_line);
    let mut args = Args::new(command_line);

    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());

    let command_line = "word \'hi\" word";
    println!("{:?}", command_line);
    let mut args = Args::new(command_line);

    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());
    println!("{:?}", args.next());
}
