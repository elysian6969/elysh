use crate::{Arg, Args, Quote, Value, Var, Vars};

#[derive(Clone, Debug)]
pub enum CommandError<'a> {
    IncompleteVar(Quote, &'a str),
    IncompleteArg(Quote, &'a str),
}

#[derive(Clone, Debug)]
pub struct Command<'a> {
    pub vars: Vec<Var<'a>>,
    pub program: Arg<'a>,
    pub args: Vec<Arg<'a>>,
    pub offset: usize,
}

impl<'a> Command<'a> {
    #[inline]
    pub fn try_parse(string: &'a str) -> Result<Self, CommandError> {
        let mut iter = Vars::new(string);
        let mut offset = 0;
        let mut vars = Vec::new();
        let mut args = Vec::new();

        while let Some(var) = iter.next() {
            match var {
                Var::Pair(_key, value) => match value {
                    //Value::IncompleteQuoted(quote, val) => {
                    //return Err(CommandError::IncompleteVar(quote, val))
                    //}
                    // Value::Quoted | Value::Word
                    // we use _ => since rustc cries about _quote not being bound in
                    // Value::Quoted(_quote, _val) | Value::Word(_val)
                    _ => {
                        offset = iter.offset();
                        vars.push(var);
                    }
                },
                _ => {}
            }
        }

        // SAFETY: `offset` is always on a character boundary and it is always a valid index
        let string = unsafe { string.get_unchecked(offset..string.len()) };
        let mut iter = Args::new(string);

        while let Some(arg) = iter.next() {
            match arg {
                Arg::Value(value) => match value {
                    //Value::IncompleteQuoted(quote, val) => {
                    //return Err(CommandError::IncompleteVar(quote, val))
                    //}
                    // Value::Quoted | Value::Word
                    // we use _ => since rustc cries about _quote not being bound in
                    // Value::Quoted(_quote, _val) | Value::Word(_val)
                    _ => {
                        offset = iter.offset();
                        args.push(arg);
                    }
                },
                _ => {}
            }
        }

        let program = if args.is_empty() {
            Arg::Value(Value::Word(""))
        } else {
            args.remove(0)
        };

        let command = Command {
            program,
            args,
            vars,
            offset,
        };

        Ok(command)
    }
}
