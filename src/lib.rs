use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Write};

pub use reveal_macro::error;

pub type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug)]
pub struct Error {
    source: Box<dyn StdError + Send + Sync>,
    context: Vec<Context>,
}

impl Error {
    pub fn context(&self) -> &Vec<Context> {
        &self.context
    }

    pub fn source(&self) -> &(dyn StdError + Send + Sync + 'static) {
        return &*self.source;
    }

    pub fn unwrap(self) -> (Box<dyn StdError + Send + Sync + 'static>, Vec<Context>) {
        (self.source, self.context)
    }

    pub fn push_context(
        &mut self,
        file: &'static str,
        line: u32,
        func: &'static str,
        args: Vec<(&'static str, String)>,
        source: &'static str,
    ) {
        self.context.push(Context {
            file,
            line,
            func,
            args,
            source,
        })
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.source, f)?;
        let mut i = 0;
        for v in &self.context {
            write!(f, "\n{}: ", i)?;
            i += 1;
            Display::fmt(v, f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Context {
    file: &'static str,
    line: u32,
    func: &'static str,
    args: Vec<(&'static str, String)>,
    source: &'static str,
}

impl Context {
    pub fn file(&self) -> &'static str {
        self.file
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn func(&self) -> &'static str {
        self.func
    }

    pub fn args(&self) -> &Vec<(&'static str, String)> {
        &self.args
    }

    pub fn source(&self) -> &'static str {
        self.source
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n\t\tin ", self.source)?;
        f.write_str(self.func)?;
        f.write_char('(')?;
        for i in 0..self.args.len() {
            write!(f, "{} = {}", self.args[i].0, self.args[i].1)?;
            if i != self.args.len() - 1 {
                f.write_str(", ")?;
            }
        }
        write!(f, ")\n\t\tat {}:{}", self.file, self.line)
    }
}

impl<T: StdError + Send + Sync + 'static> From<T> for Box<Error> {
    fn from(value: T) -> Self {
        Box::new(Error {
            source: Box::new(value),
            context: Vec::new(),
        })
    }
}
