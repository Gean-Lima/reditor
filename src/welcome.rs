use crossterm::style::Color;

pub struct WelcomeScreen;

pub struct WelcomeChar {
    pub character: char,
    pub fg_color: Color,
    pub bg_color: Color,
}

impl WelcomeScreen {
    pub fn render(columns: u16, rows: u16) -> Vec<Vec<WelcomeChar>> {
        let bg = Color::Rgb {
            r: 15,
            g: 18,
            b: 15,
        };

        let title_color = Color::Rgb {
            r: 100,
            g: 200,
            b: 130,
        };

        let shortcut_key_color = Color::Rgb {
            r: 80,
            g: 180,
            b: 220,
        };

        let shortcut_desc_color = Color::Rgb {
            r: 140,
            g: 140,
            b: 140,
        };

        let dim_color = Color::Rgb {
            r: 80,
            g: 80,
            b: 80,
        };

        let lines: Vec<(&str, Color)> = vec![
            ("", dim_color),
            (
                "██████╗ ███████╗██████╗ ██╗████████╗ ██████╗ ██████╗",
                title_color,
            ),
            (
                "██╔══██╗██╔════╝██╔══██╗██║╚══██╔══╝██╔═══██╗██╔══██╗",
                title_color,
            ),
            (
                "██████╔╝█████╗  ██║  ██║██║   ██║   ██║   ██║██████╔╝",
                title_color,
            ),
            (
                "██╔══██╗██╔══╝  ██║  ██║██║   ██║   ██║   ██║██╔══██╗",
                title_color,
            ),
            (
                "██║  ██║███████╗██████╔╝██║   ██║   ╚██████╔╝██║  ██║",
                title_color,
            ),
            (
                "╚═╝  ╚═╝╚══════╝╚═════╝ ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝",
                title_color,
            ),
            ("", dim_color),
            ("v0.1.0 — Terminal Text Editor", dim_color),
            ("", dim_color),
            ("", dim_color),
            ("Atalhos:", shortcut_desc_color),
            ("", dim_color),
            ("  Ctrl+O       Abrir arquivo", shortcut_desc_color),
            ("  Ctrl+T       Abrir/fechar sidebar", shortcut_desc_color),
            ("  Ctrl+S       Salvar arquivo", shortcut_desc_color),
            ("  Ctrl+W       Fechar aba", shortcut_desc_color),
            ("  Ctrl+Tab     Próxima aba", shortcut_desc_color),
            ("  Ctrl+F       Buscar no arquivo", shortcut_desc_color),
            ("  Ctrl+Q       Sair", shortcut_desc_color),
            ("", dim_color),
            ("  i            Modo Insert", shortcut_desc_color),
            ("  Esc          Modo Normal", shortcut_desc_color),
            ("  Home/End     Início/fim da linha", shortcut_desc_color),
            ("", dim_color),
            ("", dim_color),
            ("  Use: reditor <arquivo|pasta>", dim_color),
        ];

        let mut matrix: Vec<Vec<WelcomeChar>> = vec![];
        let start_row = (rows as usize).saturating_sub(lines.len()) / 2;

        for row in 0..rows as usize {
            let mut row_chars: Vec<WelcomeChar> = vec![];
            let line_idx = row.checked_sub(start_row);

            let (line_text, line_color) = if let Some(idx) = line_idx {
                if idx < lines.len() {
                    lines[idx]
                } else {
                    ("", dim_color)
                }
            } else {
                ("", dim_color)
            };

            // Center the line
            let line_chars: Vec<char> = line_text.chars().collect();
            let char_count = line_chars.len();
            let padding = (columns as usize).saturating_sub(char_count) / 2;

            for col in 0..columns as usize {
                let ch = if col >= padding && col < padding + char_count {
                    let ch = line_chars[col - padding];
                    // Color shortcut keys differently
                    let fg = if line_text.starts_with("  Ctrl+")
                        || line_text.starts_with("  i ")
                        || line_text.starts_with("  Esc")
                        || line_text.starts_with("  Home")
                    {
                        if col - padding < 14 {
                            shortcut_key_color
                        } else {
                            line_color
                        }
                    } else {
                        line_color
                    };
                    WelcomeChar {
                        character: ch,
                        fg_color: fg,
                        bg_color: bg,
                    }
                } else {
                    WelcomeChar {
                        character: ' ',
                        fg_color: dim_color,
                        bg_color: bg,
                    }
                };
                row_chars.push(ch);
            }

            matrix.push(row_chars);
        }

        matrix
    }
}
