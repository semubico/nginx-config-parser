use std::fmt::Display;

use logos::Logos;
mod types;

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

    #[regex(r#"(\w|\d|\[|\]|=|\^|\$|:|-|\+|!|\*|~|\'|\.|\/)+"#)]
    Word(&'a str),
}

impl<'l> Display for Token<'l> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment(c) => write!(f, "# {}", c.trim()),
            Self::BracketOpen => write!(f, "{{"),
            Self::BracketClose => write!(f, "}}"),
            Self::Semicolon => write!(f, ";"),
            Self::BracedString(s) => write!(f, "({})", s),
            Self::QuotedString(s) => write!(f, "\"{}\"", s),
            Self::Word(s) => write!(f, "{}", s),
            Self::Newline => write!(f, "\n"),
        }
    }
}

/// Statements are one-line, ';'-terminated directives
/// Blocks are nulti-line, '{' and '}' enclosing directives
/// Content of the directive preceding the ';'/'{' is stored in the `args` field as a `Vec<String>`
/// The directive name is stored under args[0]
#[derive(Debug, Clone)]
pub enum Structure<'l> {
    Statement {
        args: Vec<Token<'l>>,
    },
    Block {
        args: Vec<Token<'l>>,
        children: Vec<Structure<'l>>,
    },
}

impl<'l> Structure<'l> {
    pub fn args(&mut self) -> &mut Vec<Token<'l>> {
        match self {
            Self::Statement { args } => args,
            Self::Block { args, .. } => args,
        }
    }
    pub fn children(&mut self) -> &mut Vec<Structure<'l>> {
        match self {
            Self::Block { children, .. } => children,
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
    pub fn parse(cfg: &'l str) -> Result<Self, String> {
        let mut lex = Token::lexer(&cfg).spanned();
        let mut stack = Vec::new();
        let mut current_block = Self::Block {
            args: Vec::new(),
            children: Vec::new(),
        };

        let mut current_statement = Self::Statement { args: Vec::new() };

        loop {
            let token = match lex.next() {
                Some((Ok(token), _)) => token,
                Some((Err(()), span)) => return Err(format!("{:?}", span)),
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

                Token::QuotedString(_content) => { current_statement.args().push(token); },
                Token::BracedString(_content) => { current_statement.args().push(token); },
                Token::Word(_content) => { current_statement.args().push(token); }
            }
        }
        
        if let Self::Statement { args } = current_statement {
            current_block.args().extend(args.clone());        
        };       

        Ok(current_block)
    }
}


