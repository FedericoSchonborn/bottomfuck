use std::{
    env,
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, Read, Write},
    iter::Peekable,
    num::Wrapping,
    str::Chars,
};

#[derive(Debug, Clone)]
enum Command {
    Add,
    Sub,
    Input,
    Output,
    Left,
    Right,
    Loop,
    Pool,
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
        }
    }
}

#[derive(Debug)]
enum ParserError {
    InvalidFingers,
    IncompleteHeart,
}

impl Error for ParserError {}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFingers => "missing matching 👈".fmt(f),
            Self::IncompleteHeart => "incomplete ❤️".fmt(f),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Command, ParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Ok(match self.chars.next()? {
            '❤' => {
                if let Some('\u{fe0f}') = self.chars.next() {
                    Command::Left
                } else {
                    return Some(Err(ParserError::IncompleteHeart));
                }
            }
            '💖' => Command::Right,
            '👉' => {
                if let Some('👈') = self.chars.next() {
                    Command::Add
                } else {
                    return Some(Err(ParserError::InvalidFingers));
                }
            }
            '🥺' => Command::Sub,
            ',' => Command::Input,
            '.' => Command::Output,
            '🫂' => Command::Loop,
            '✨' => Command::Pool,
            _ => return self.next(),
        }))
    }
}

struct Machine<R, W> {
    memory: [Wrapping<u8>; 30_000],
    pointer: Wrapping<usize>,
    input: R,
    output: W,
}

#[derive(Debug)]
enum MachineError {
    UnexpectedEof,
    UnmatchedPool,
    Io(io::Error),
}

impl Error for MachineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnexpectedEof | Self::UnmatchedPool => None,
            Self::Io(err) => Some(err),
        }
    }
}

impl Display for MachineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => "unexpected end of file".fmt(f),
            Self::UnmatchedPool => "unmatched ✨".fmt(f),
            Self::Io(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for MachineError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl<R, W> Machine<R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(input: R, output: W) -> Self {
        Self {
            memory: [Wrapping(0); 30_000],
            pointer: Wrapping(0),
            input,
            output,
        }
    }

    pub fn run(&mut self, program: &[Command]) -> Result<(), MachineError> {
        for (index, command) in program.iter().enumerate() {
            match dbg!(command) {
                Command::Left => self.pointer -= 1,
                Command::Right => self.pointer += 1,
                Command::Add => self.memory[self.pointer.0] += 1,
                Command::Sub => self.memory[self.pointer.0] -= 1,
                Command::Input => {
                    let mut buffer = [0u8; 1];
                    self.input.read_exact(&mut buffer)?;

                    self.memory[self.pointer.0] = Wrapping(buffer[0]);
                }
                Command::Output => {
                    write!(self.output, "{}", char::from(self.memory[self.pointer.0].0))?;
                }
                Command::Loop => {
                    let start = index + 1;
                    let mut end = start;

                    let mut level = 1;
                    loop {
                        match program.get(end) {
                            Some(Command::Loop) => level += 1,
                            Some(Command::Pool) => {
                                level -= 1;
                                if level == 0 {
                                    break;
                                }
                            }
                            Some(_) => {}
                            None => return Err(MachineError::UnexpectedEof),
                        }

                        end += 1;
                    }

                    while self.memory[self.pointer.0].0 != 0 {
                        self.run(&program[start..end])?;
                    }
                }
                Command::Pool => return Err(MachineError::UnmatchedPool),
            }

            println!("{:?}", &self.memory[0..9]);
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args().nth(1).expect("missing path argument");
    let source = fs::read_to_string(path)?;
    let parser = Parser::new(&source);
    let program = parser.collect::<Result<Vec<_>, _>>()?;
    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let stdout_lock = stdout.lock();
    let mut machine = Machine::new(stdin_lock, stdout_lock);
    machine.run(&program)?;

    Ok(())
}
