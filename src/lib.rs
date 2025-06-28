use logos::Lexer;
use logos::Logos;

#[derive(Debug, Logos, Copy, Clone)]
#[logos(skip r"[ \t]+", error = ())]
pub enum Token<'a> {
    #[regex(r"#.*\n")]
    Comment(&'a str),

    #[token("{")]
    BracketOpen,

    #[token("\n")]
    Newline,

    #[token("}")]
    BracketClose,

    #[token(";", priority = 3)]
    Semicolon,

    #[regex(r#"\([^\)]+\)"#)]
    BracedString(&'a str),

    #[regex(r#"\"[^\"]*\""#, priority = 4)]
    QuotedString(&'a str),

    #[regex(r#"(\w|\d|\[|\]|=|\^|\$|:|-|!|\*|~|\'|\.|\/)+"#)]
    Word(&'a str),
}

/// Statements are one-line, ';'-terminated directives
/// Blocks are nulti-line, '{' and '}' enclosing directives
/// Content of the directive preceding the ';'/'{' is stored in the `args` field as a `Vec<String>`
/// The directive name is stored under args[0]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Structure {
    Statement {
        args: Vec<String>,
    },
    Block {
        args: Vec<String>,
        children: Vec<Structure>,
    },
}

impl Structure {
    pub fn args(&mut self) -> &mut Vec<String> {
        match self {
            Self::Statement { args } => args.as_mut(),
            Self::Block { args, .. } => args.as_mut(),
        }
    }
    pub fn children(&mut self) -> &mut Vec<Structure> {
        match self {
            Self::Block { children, .. } => children.as_mut(),
            _ => unreachable!(),
        }
    }

    /// @warning 
    /// ```
    /// This is a dumb parser used to read existing nginx configs
    /// It doesn't validate the configuration
    /// ```
    /// @example
    /// ```
    ///    let cfg = std::fs::read_to_string("./example.conf").unwrap();
    ///    let cfg = Structure::parse(cfg);
    ///```
    ///  
    pub fn parse<'a>(cfg: &'a str) -> Result<Self, ()> {
        let mut lex = Token::lexer(&cfg);
        let mut stack = Vec::new();
        let mut current_block = Self::Block {
            args: Vec::new(),
            children: Vec::new(),
        };

        let mut current_statement = Self::Statement { args: Vec::new() };

        loop {
            let token = match lex.next() {
                Some(Ok(token)) => token,
                Some(Err(())) => return Err(()),
                None => break,
            };

            match token {
                Token::BracketOpen => {
                    stack.push(current_block.clone());
                    current_block = Self::Block {
                        args: current_statement
                            .args()
                            .iter()
                            .map(|v| v.clone())
                            .collect::<Vec<_>>(),
                        children: Vec::new(),
                    };
                    current_statement = Self::Statement { args: Vec::new() };
                }

                Token::BracketClose => {
                    if current_statement.args().len() > 0 {
                        current_block.children().push(Self::Statement {
                            args: current_statement.args().clone(),
                        });
                    }
                    if let Some(parent_block) = stack.pop() {
                        if let Self::Block { args, mut children } = parent_block {
                            children.push(current_block.clone());
                            stack.push(Self::Block {
                                args: args.clone(),
                                children: children.clone(),
                            });
                            current_block = Self::Block {
                                args: args,
                                children: children.clone(),
                            }
                        }
                    }
                    current_statement = Self::Statement { args: Vec::new() };
                }

                Token::Semicolon => {
                    current_block.children().push(Self::Statement {
                        args: current_statement.args().clone(),
                    });
                    current_statement = Self::Statement { args: Vec::new() };
                }

                Token::Comment(_pat) => {}

                Token::Newline => {}

                Token::QuotedString(content)
                | Token::BracedString(content)
                | Token::Word(content) => {
                    current_statement.args().push(content.to_string());
                }
            }
        }

        if let Self::Statement { args } = current_statement {
            current_block.args().extend(args);
        }

        Ok(current_block)
    }
}
