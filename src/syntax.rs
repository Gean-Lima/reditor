use crossterm::style::Color;

/// A single colored character for display
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct ColoredChar {
    pub ch: char,
    pub fg: Color,
}

/// Token types for syntax highlighting
#[derive(Clone, Copy, PartialEq)]
enum TokenType {
    Normal,
    Keyword,
    String,
    Comment,
    Number,
    Type,
    Function,
    Operator,
    Punctuation,
    Attribute,
    Macro,
    Lifetime,
}

/// Colors for each token type
fn token_color(tt: TokenType) -> Color {
    match tt {
        TokenType::Normal => Color::Rgb {
            r: 200,
            g: 200,
            b: 200,
        },
        TokenType::Keyword => Color::Rgb {
            r: 198,
            g: 120,
            b: 221,
        }, // purple
        TokenType::String => Color::Rgb {
            r: 152,
            g: 195,
            b: 121,
        }, // green
        TokenType::Comment => Color::Rgb {
            r: 92,
            g: 99,
            b: 112,
        }, // gray
        TokenType::Number => Color::Rgb {
            r: 209,
            g: 154,
            b: 102,
        }, // orange
        TokenType::Type => Color::Rgb {
            r: 229,
            g: 192,
            b: 123,
        }, // yellow
        TokenType::Function => Color::Rgb {
            r: 97,
            g: 175,
            b: 239,
        }, // blue
        TokenType::Operator => Color::Rgb {
            r: 86,
            g: 182,
            b: 194,
        }, // cyan
        TokenType::Punctuation => Color::Rgb {
            r: 171,
            g: 178,
            b: 191,
        }, // light gray
        TokenType::Attribute => Color::Rgb {
            r: 229,
            g: 192,
            b: 123,
        }, // yellow
        TokenType::Macro => Color::Rgb {
            r: 86,
            g: 182,
            b: 194,
        }, // cyan
        TokenType::Lifetime => Color::Rgb {
            r: 209,
            g: 154,
            b: 102,
        }, // orange
    }
}

/// Language definition
struct Language {
    keywords: &'static [&'static str],
    types: &'static [&'static str],
    line_comment: &'static str,
    block_comment_start: &'static str,
    block_comment_end: &'static str,
    has_macros: bool,
    has_lifetimes: bool,
}

fn language_for_ext(ext: &str) -> Option<Language> {
    match ext {
        "rs" => Some(Language {
            keywords: &[
                "fn", "let", "mut", "const", "static", "if", "else", "match", "for", "while",
                "loop", "return", "break", "continue", "struct", "enum", "impl", "trait", "type",
                "pub", "mod", "use", "crate", "self", "super", "as", "in", "ref", "move", "where",
                "async", "await", "dyn", "unsafe", "extern", "true", "false",
            ],
            types: &[
                "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
                "isize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Self", "Ok", "Err", "Some", "None", "HashMap", "HashSet",
                "BTreeMap", "BTreeSet", "PathBuf", "Path", "io", "fs", "fmt", "Display",
            ],
            line_comment: "//",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: true,
            has_lifetimes: true,
        }),
        "js" | "jsx" | "ts" | "tsx" | "mjs" => Some(Language {
            keywords: &[
                "function",
                "var",
                "let",
                "const",
                "if",
                "else",
                "for",
                "while",
                "do",
                "switch",
                "case",
                "break",
                "continue",
                "return",
                "new",
                "delete",
                "typeof",
                "instanceof",
                "in",
                "of",
                "class",
                "extends",
                "super",
                "this",
                "import",
                "export",
                "default",
                "from",
                "as",
                "try",
                "catch",
                "finally",
                "throw",
                "async",
                "await",
                "yield",
                "true",
                "false",
                "null",
                "undefined",
                "void",
            ],
            types: &[
                "Array",
                "Object",
                "String",
                "Number",
                "Boolean",
                "Map",
                "Set",
                "Promise",
                "Date",
                "RegExp",
                "Error",
                "JSON",
                "Math",
                "console",
                "window",
                "document",
                "any",
                "string",
                "number",
                "boolean",
                "never",
                "unknown",
                "interface",
                "type",
                "enum",
            ],
            line_comment: "//",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        "py" => Some(Language {
            keywords: &[
                "def", "class", "if", "elif", "else", "for", "while", "break", "continue",
                "return", "pass", "import", "from", "as", "try", "except", "finally", "raise",
                "with", "yield", "lambda", "and", "or", "not", "in", "is", "global", "nonlocal",
                "assert", "del", "True", "False", "None", "async", "await",
            ],
            types: &[
                "int",
                "float",
                "str",
                "bool",
                "list",
                "dict",
                "tuple",
                "set",
                "bytes",
                "type",
                "object",
                "range",
                "print",
                "len",
                "self",
                "super",
                "Exception",
            ],
            line_comment: "#",
            block_comment_start: "",
            block_comment_end: "",
            has_macros: false,
            has_lifetimes: false,
        }),
        "c" | "h" => Some(Language {
            keywords: &[
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "int", "long", "register",
                "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef",
                "union", "unsigned", "void", "volatile", "while", "inline", "restrict", "NULL",
                "true", "false",
            ],
            types: &[
                "int", "char", "float", "double", "long", "short", "unsigned", "signed", "void",
                "size_t", "uint8_t", "uint16_t", "uint32_t", "uint64_t", "int8_t", "int16_t",
                "int32_t", "int64_t", "FILE", "bool",
            ],
            line_comment: "//",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        "cpp" | "cc" | "cxx" | "hpp" => Some(Language {
            keywords: &[
                "auto",
                "break",
                "case",
                "catch",
                "class",
                "const",
                "continue",
                "default",
                "delete",
                "do",
                "else",
                "enum",
                "extern",
                "for",
                "friend",
                "goto",
                "if",
                "inline",
                "namespace",
                "new",
                "operator",
                "private",
                "protected",
                "public",
                "return",
                "sizeof",
                "static",
                "struct",
                "switch",
                "template",
                "this",
                "throw",
                "try",
                "typedef",
                "union",
                "using",
                "virtual",
                "void",
                "volatile",
                "while",
                "override",
                "final",
                "noexcept",
                "constexpr",
                "nullptr",
                "true",
                "false",
            ],
            types: &[
                "int",
                "char",
                "float",
                "double",
                "long",
                "short",
                "unsigned",
                "signed",
                "void",
                "bool",
                "string",
                "vector",
                "map",
                "set",
                "pair",
                "size_t",
                "wchar_t",
                "unique_ptr",
                "shared_ptr",
                "weak_ptr",
                "std",
            ],
            line_comment: "//",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        "go" => Some(Language {
            keywords: &[
                "break",
                "case",
                "chan",
                "const",
                "continue",
                "default",
                "defer",
                "else",
                "fallthrough",
                "for",
                "func",
                "go",
                "goto",
                "if",
                "import",
                "interface",
                "map",
                "package",
                "range",
                "return",
                "select",
                "struct",
                "switch",
                "type",
                "var",
                "true",
                "false",
                "nil",
            ],
            types: &[
                "int",
                "int8",
                "int16",
                "int32",
                "int64",
                "uint",
                "uint8",
                "uint16",
                "uint32",
                "uint64",
                "float32",
                "float64",
                "complex64",
                "complex128",
                "byte",
                "rune",
                "string",
                "bool",
                "error",
                "any",
            ],
            line_comment: "//",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        "java" | "kt" | "kts" => Some(Language {
            keywords: &[
                "abstract",
                "assert",
                "boolean",
                "break",
                "byte",
                "case",
                "catch",
                "char",
                "class",
                "const",
                "continue",
                "default",
                "do",
                "double",
                "else",
                "enum",
                "extends",
                "final",
                "finally",
                "float",
                "for",
                "goto",
                "if",
                "implements",
                "import",
                "instanceof",
                "int",
                "interface",
                "long",
                "native",
                "new",
                "package",
                "private",
                "protected",
                "public",
                "return",
                "short",
                "static",
                "strictfp",
                "super",
                "switch",
                "synchronized",
                "this",
                "throw",
                "throws",
                "transient",
                "try",
                "void",
                "volatile",
                "while",
                "true",
                "false",
                "null",
            ],
            types: &[
                "String",
                "Integer",
                "Boolean",
                "Double",
                "Float",
                "Long",
                "Short",
                "Byte",
                "Character",
                "Object",
                "List",
                "Map",
                "Set",
                "ArrayList",
                "HashMap",
                "HashSet",
                "Optional",
                "Stream",
                "var",
                "val",
            ],
            line_comment: "//",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        "toml" => Some(Language {
            keywords: &["true", "false"],
            types: &[],
            line_comment: "#",
            block_comment_start: "",
            block_comment_end: "",
            has_macros: false,
            has_lifetimes: false,
        }),
        "yaml" | "yml" => Some(Language {
            keywords: &["true", "false", "null", "yes", "no", "on", "off"],
            types: &[],
            line_comment: "#",
            block_comment_start: "",
            block_comment_end: "",
            has_macros: false,
            has_lifetimes: false,
        }),
        "sh" | "bash" | "zsh" => Some(Language {
            keywords: &[
                "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac",
                "function", "return", "exit", "echo", "read", "local", "export", "source", "set",
                "unset", "shift", "true", "false",
            ],
            types: &[],
            line_comment: "#",
            block_comment_start: "",
            block_comment_end: "",
            has_macros: false,
            has_lifetimes: false,
        }),
        "css" | "scss" | "sass" => Some(Language {
            keywords: &[
                "import",
                "media",
                "keyframes",
                "font-face",
                "charset",
                "supports",
                "namespace",
                "page",
                "important",
                "from",
                "to",
            ],
            types: &[],
            line_comment: "",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        "html" | "htm" | "xml" | "svg" => Some(Language {
            keywords: &[],
            types: &[],
            line_comment: "",
            block_comment_start: "<!--",
            block_comment_end: "-->",
            has_macros: false,
            has_lifetimes: false,
        }),
        "json" => Some(Language {
            keywords: &["true", "false", "null"],
            types: &[],
            line_comment: "",
            block_comment_start: "",
            block_comment_end: "",
            has_macros: false,
            has_lifetimes: false,
        }),
        "md" | "markdown" => Some(Language {
            keywords: &[],
            types: &[],
            line_comment: "",
            block_comment_start: "",
            block_comment_end: "",
            has_macros: false,
            has_lifetimes: false,
        }),
        "sql" => Some(Language {
            keywords: &[
                "SELECT",
                "FROM",
                "WHERE",
                "INSERT",
                "UPDATE",
                "DELETE",
                "CREATE",
                "DROP",
                "ALTER",
                "TABLE",
                "INDEX",
                "VIEW",
                "INTO",
                "VALUES",
                "SET",
                "JOIN",
                "LEFT",
                "RIGHT",
                "INNER",
                "OUTER",
                "ON",
                "AND",
                "OR",
                "NOT",
                "NULL",
                "IS",
                "IN",
                "LIKE",
                "BETWEEN",
                "ORDER",
                "BY",
                "GROUP",
                "HAVING",
                "LIMIT",
                "OFFSET",
                "UNION",
                "ALL",
                "AS",
                "DISTINCT",
                "EXISTS",
                "CASE",
                "WHEN",
                "THEN",
                "ELSE",
                "END",
                "PRIMARY",
                "KEY",
                "FOREIGN",
                "REFERENCES",
                "TRUE",
                "FALSE",
                "select",
                "from",
                "where",
                "insert",
                "update",
                "delete",
                "create",
                "drop",
                "alter",
                "table",
                "index",
                "view",
                "into",
                "values",
                "set",
                "join",
                "left",
                "right",
                "inner",
                "outer",
                "on",
                "and",
                "or",
                "not",
                "null",
                "is",
                "in",
                "like",
                "between",
                "order",
                "by",
                "group",
                "having",
                "limit",
                "offset",
                "union",
                "all",
                "as",
                "distinct",
                "exists",
                "case",
                "when",
                "then",
                "else",
                "end",
                "primary",
                "key",
                "foreign",
                "references",
                "true",
                "false",
            ],
            types: &[
                "INT",
                "INTEGER",
                "BIGINT",
                "SMALLINT",
                "FLOAT",
                "DOUBLE",
                "VARCHAR",
                "CHAR",
                "TEXT",
                "BOOLEAN",
                "DATE",
                "TIMESTAMP",
                "BLOB",
                "DECIMAL",
                "NUMERIC",
            ],
            line_comment: "--",
            block_comment_start: "/*",
            block_comment_end: "*/",
            has_macros: false,
            has_lifetimes: false,
        }),
        _ => None,
    }
}

/// State carried between lines for multi-line constructs
#[derive(Clone, Copy)]
pub struct HighlightState {
    pub in_block_comment: bool,
}

impl HighlightState {
    pub fn new() -> Self {
        HighlightState {
            in_block_comment: false,
        }
    }
}

/// Highlight a single line given a language extension and carry-over state.
/// Returns (colored chars, updated state).
pub fn highlight_line(line: &[char], ext: &str, state: &mut HighlightState) -> Vec<ColoredChar> {
    let lang = match language_for_ext(ext) {
        Some(l) => l,
        None => {
            // No highlighting — return all normal
            return line
                .iter()
                .map(|&ch| ColoredChar {
                    ch,
                    fg: token_color(TokenType::Normal),
                })
                .collect();
        }
    };

    let len = line.len();
    let mut result: Vec<ColoredChar> = Vec::with_capacity(len);
    let mut i = 0;

    let lc_chars: Vec<char> = lang.line_comment.chars().collect();
    let bc_start: Vec<char> = lang.block_comment_start.chars().collect();
    let bc_end: Vec<char> = lang.block_comment_end.chars().collect();

    while i < len {
        // --- Block comment continuation ---
        if state.in_block_comment {
            if !bc_end.is_empty() && starts_with_at(line, i, &bc_end) {
                for &ch in &bc_end {
                    result.push(ColoredChar {
                        ch,
                        fg: token_color(TokenType::Comment),
                    });
                }
                i += bc_end.len();
                state.in_block_comment = false;
            } else {
                result.push(ColoredChar {
                    ch: line[i],
                    fg: token_color(TokenType::Comment),
                });
                i += 1;
            }
            continue;
        }

        // --- Block comment start ---
        if !bc_start.is_empty() && starts_with_at(line, i, &bc_start) {
            state.in_block_comment = true;
            for &ch in &bc_start {
                result.push(ColoredChar {
                    ch,
                    fg: token_color(TokenType::Comment),
                });
            }
            i += bc_start.len();
            continue;
        }

        // --- Line comment ---
        if !lc_chars.is_empty() && starts_with_at(line, i, &lc_chars) {
            while i < len {
                result.push(ColoredChar {
                    ch: line[i],
                    fg: token_color(TokenType::Comment),
                });
                i += 1;
            }
            break;
        }

        // --- Strings (double-quoted) ---
        if line[i] == '"' {
            result.push(ColoredChar {
                ch: '"',
                fg: token_color(TokenType::String),
            });
            i += 1;
            while i < len {
                let ch = line[i];
                result.push(ColoredChar {
                    ch,
                    fg: token_color(TokenType::String),
                });
                i += 1;
                if ch == '\\' && i < len {
                    result.push(ColoredChar {
                        ch: line[i],
                        fg: token_color(TokenType::String),
                    });
                    i += 1;
                } else if ch == '"' {
                    break;
                }
            }
            continue;
        }

        // --- Strings (single-quoted) ---
        if line[i] == '\'' {
            // In Rust, check for lifetime: 'a, 'static, etc.
            if lang.has_lifetimes && i + 1 < len && line[i + 1].is_alphabetic() {
                // Could be a lifetime
                let start_pos = i;
                i += 1; // skip '
                let word_start = i;
                while i < len && (line[i].is_alphanumeric() || line[i] == '_') {
                    i += 1;
                }
                // If it's a short word without closing quote, it's a lifetime
                let word: String = line[word_start..i].iter().collect();
                if i >= len || line[i] != '\'' || word.len() <= 10 {
                    // Lifetime
                    result.push(ColoredChar {
                        ch: '\'',
                        fg: token_color(TokenType::Lifetime),
                    });
                    for ch in word.chars() {
                        result.push(ColoredChar {
                            ch,
                            fg: token_color(TokenType::Lifetime),
                        });
                    }
                    continue;
                } else {
                    // char literal: 'x'
                    i = start_pos; // reset
                }
            }

            result.push(ColoredChar {
                ch: '\'',
                fg: token_color(TokenType::String),
            });
            i += 1;
            while i < len {
                let ch = line[i];
                result.push(ColoredChar {
                    ch,
                    fg: token_color(TokenType::String),
                });
                i += 1;
                if ch == '\\' && i < len {
                    result.push(ColoredChar {
                        ch: line[i],
                        fg: token_color(TokenType::String),
                    });
                    i += 1;
                } else if ch == '\'' {
                    break;
                }
            }
            continue;
        }

        // --- Backtick strings (JS template literals) ---
        if line[i] == '`' {
            result.push(ColoredChar {
                ch: '`',
                fg: token_color(TokenType::String),
            });
            i += 1;
            while i < len {
                let ch = line[i];
                result.push(ColoredChar {
                    ch,
                    fg: token_color(TokenType::String),
                });
                i += 1;
                if ch == '\\' && i < len {
                    result.push(ColoredChar {
                        ch: line[i],
                        fg: token_color(TokenType::String),
                    });
                    i += 1;
                } else if ch == '`' {
                    break;
                }
            }
            continue;
        }

        // --- Rust attributes: #[...] or #![...] ---
        if lang.has_macros
            && line[i] == '#'
            && i + 1 < len
            && (line[i + 1] == '[' || line[i + 1] == '!')
        {
            while i < len && line[i] != ']' {
                result.push(ColoredChar {
                    ch: line[i],
                    fg: token_color(TokenType::Attribute),
                });
                i += 1;
            }
            if i < len {
                result.push(ColoredChar {
                    ch: line[i],
                    fg: token_color(TokenType::Attribute),
                });
                i += 1;
            }
            continue;
        }

        // --- Numbers ---
        if line[i].is_ascii_digit()
            || (line[i] == '.'
                && i + 1 < len
                && line[i + 1].is_ascii_digit()
                && (i == 0 || !line[i - 1].is_alphanumeric()))
        {
            while i < len
                && (line[i].is_ascii_digit()
                    || line[i] == '.'
                    || line[i] == 'x'
                    || line[i] == 'b'
                    || line[i] == 'o'
                    || line[i] == '_'
                    || (line[i] >= 'a' && line[i] <= 'f')
                    || (line[i] >= 'A' && line[i] <= 'F'))
            {
                result.push(ColoredChar {
                    ch: line[i],
                    fg: token_color(TokenType::Number),
                });
                i += 1;
            }
            continue;
        }

        // --- Identifiers / Keywords ---
        if line[i].is_alphabetic() || line[i] == '_' {
            let start = i;
            while i < len && (line[i].is_alphanumeric() || line[i] == '_') {
                i += 1;
            }
            let word: String = line[start..i].iter().collect();

            // Check for macro invocation: word!
            if lang.has_macros && i < len && line[i] == '!' {
                for ch in word.chars() {
                    result.push(ColoredChar {
                        ch,
                        fg: token_color(TokenType::Macro),
                    });
                }
                result.push(ColoredChar {
                    ch: '!',
                    fg: token_color(TokenType::Macro),
                });
                i += 1;
                continue;
            }

            // Check for function call: word(
            let is_fn_call = i < len && line[i] == '(';

            let tt = if lang.keywords.contains(&word.as_str()) {
                TokenType::Keyword
            } else if lang.types.contains(&word.as_str()) {
                TokenType::Type
            } else if is_fn_call {
                TokenType::Function
            } else if word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                TokenType::Type // PascalCase = type
            } else {
                TokenType::Normal
            };

            for ch in word.chars() {
                result.push(ColoredChar {
                    ch,
                    fg: token_color(tt),
                });
            }
            continue;
        }

        // --- Operators ---
        if "=+-*/<>!&|^%~?:".contains(line[i]) {
            result.push(ColoredChar {
                ch: line[i],
                fg: token_color(TokenType::Operator),
            });
            i += 1;
            continue;
        }

        // --- Punctuation ---
        if "(){}[];,.@".contains(line[i]) {
            result.push(ColoredChar {
                ch: line[i],
                fg: token_color(TokenType::Punctuation),
            });
            i += 1;
            continue;
        }

        // --- Everything else ---
        result.push(ColoredChar {
            ch: line[i],
            fg: token_color(TokenType::Normal),
        });
        i += 1;
    }

    result
}

fn starts_with_at(line: &[char], pos: usize, pattern: &[char]) -> bool {
    if pos + pattern.len() > line.len() {
        return false;
    }
    for (j, &pch) in pattern.iter().enumerate() {
        if line[pos + j] != pch {
            return false;
        }
    }
    true
}

/// Get a file icon based on extension
pub fn file_icon(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "🦀",
        "js" | "mjs" => "🟨",
        "ts" => "🔷",
        "jsx" | "tsx" => "⚛️",
        "py" => "🐍",
        "rb" => "💎",
        "go" => "🔹",
        "java" => "☕",
        "kt" | "kts" => "🟪",
        "c" | "h" => "🔧",
        "cpp" | "cc" | "cxx" | "hpp" => "⚙️",
        "cs" => "🟩",
        "swift" => "🐦",
        "php" => "🐘",
        "html" | "htm" => "🌐",
        "css" => "🎨",
        "scss" | "sass" | "less" => "🎨",
        "json" => "📋",
        "xml" | "svg" => "📄",
        "yaml" | "yml" => "⚙️",
        "toml" => "⚙️",
        "md" | "markdown" => "📝",
        "txt" => "📄",
        "sh" | "bash" | "zsh" => "🖥️",
        "sql" => "🗃️",
        "dockerfile" | "docker" => "🐳",
        "git" | "gitignore" => "🔀",
        "lock" => "🔒",
        "env" => "🔐",
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp" => "🖼️",
        "mp3" | "wav" | "ogg" | "flac" => "🎵",
        "mp4" | "avi" | "mov" | "mkv" | "webm" => "🎬",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" => "📦",
        "pdf" => "📕",
        "wasm" => "🌀",
        _ => {
            // Check for special filenames
            let lower = filename.to_lowercase();
            if lower == "cargo.toml" || lower == "cargo.lock" {
                "📦"
            } else if lower == "makefile" || lower == "cmakeLists.txt" {
                "🔨"
            } else if lower == "readme" || lower.starts_with("readme.") {
                "📖"
            } else if lower == "license" || lower.starts_with("license") {
                "⚖️"
            } else {
                "📄"
            }
        }
    }
}

/// Get the file extension from a filename/path
pub fn get_extension(filename: &str) -> String {
    if let Some(pos) = filename.rfind('.') {
        filename[pos + 1..].to_lowercase()
    } else {
        String::new()
    }
}
