use elysh_syntax::Command;

fn main() {
    let command = "LD_DEBUG=\"1\" ls -aFhl";
    let command = Command::try_parse(command);

    println!("{:?}", command);

    let command = "LD_DEBUG=\"1 ls -aFhl";
    let command = Command::try_parse(command);

    println!("{:?}", command);

    let command = "LD_DEBUG=\"1\" ls -aFhl \"path with space\" path_without_space";
    let command = Command::try_parse(command);

    println!("{:?}", command);

    let command = "LD_DEBUG=\"1\" ls -aFhl \"path with space path_without_space";
    let command = Command::try_parse(command);

    println!("{:?}", command);
}
