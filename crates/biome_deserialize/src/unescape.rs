use crate::DeserializationDiagnostic;
use biome_console::markup;

#[derive(Copy, Clone)]
enum EscapeState {
    Normal,
    // We just saw a `\`
    Escape,
    // We just saw a `\u` and now expect 4 hex digits
    Unicode,
}

pub fn unescape_str(s: &str) -> Result<String, DeserializationDiagnostic> {
    let mut state = EscapeState::Normal;
    // Invariant: unicode_digits.len() < 4 at the end of the loop
    let mut unicode_digits = String::new();
    let mut out = String::new();
    for c in s.chars() {
        match (c, state) {
            ('\\' | '"' | '/', EscapeState::Escape) => {
                out.push(c);
                state = EscapeState::Normal;
            }
            ('b', EscapeState::Escape) => out.push('\x08'),
            ('f', EscapeState::Escape) => out.push('\x0c'),
            ('n', EscapeState::Escape) => out.push('\n'),
            ('r', EscapeState::Escape) => out.push('\r'),
            ('t', EscapeState::Escape) => out.push('\t'),
            ('u', EscapeState::Escape) => {
                unicode_digits = String::new();
                state = EscapeState::Unicode;
            }
            (c, EscapeState::Escape) => {
                return Err(DeserializationDiagnostic::new(
                    markup!("The escape sequence `"<Emphasis>{"\\"}{c}</Emphasis>"` is invalid."),
                ));
            }
            (c, EscapeState::Unicode) if c.is_ascii_hexdigit() => {
                unicode_digits.push(c);
            }
            (c, EscapeState::Unicode) => {
                return Err(DeserializationDiagnostic::new(
                    markup!("The unicode escape sequence `"<Emphasis>{"\\u"}{unicode_digits}{c}</Emphasis>"` is invalid (`"{c}"` is not a valid hexadecimal character."),
                ));
            }
            ('\\', EscapeState::Normal) => state = EscapeState::Escape,
            (c, EscapeState::Normal) => out.push(c),
        }

        if unicode_digits.len() == 4 {
            let codepoint = u32::from_str_radix(&unicode_digits, 16).map_err(|_| {
                DeserializationDiagnostic::new(markup!("The unicode escape sequence `"<Emphasis>{"\\u"}{unicode_digits}</Emphasis>"` is invalid (it is not a valid unicode codepoint)."))
            })?;
            out.push(char::from_u32(codepoint).ok_or_else(|| {
                DeserializationDiagnostic::new(markup!("The unicode escape sequence `"<Emphasis>{"\\u"}{unicode_digits}</Emphasis>"` is invalid (it is not a valid unicode codepoint)."))
            })?);
            unicode_digits = String::new();
            state = EscapeState::Normal;
        }
    }

    match state {
        EscapeState::Normal => Ok(out),
        EscapeState::Escape => Err(DeserializationDiagnostic::new(markup!(
            "Reached end of input, expected escape sequence."
        ))),
        EscapeState::Unicode => Err(DeserializationDiagnostic::new(markup!(
            "Reached end of input, expected unicode escape sequence."
        ))),
    }
}
