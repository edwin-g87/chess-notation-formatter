//! Normalizes messy chess movetext into a consistent style:
//! move numbers followed by ". " (or "... " for a black move shown on its
//! own), uppercase piece letters, "x" for captures instead of ":", and "#"
//! instead of "++" for mate. Lines that look like PGN tag pairs (starting
//! with '[') are passed through untouched.

pub fn normalize(input: &str) -> String {
    let mut out = String::new();
    for line in input.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim_start().starts_with('[') {
            out.push_str(trimmed);
        } else {
            out.push_str(&normalize_line(trimmed));
        }
        out.push('\n');
    }
    out
}

fn normalize_line(line: &str) -> String {
    line.split_whitespace()
        .map(normalize_token)
        .collect::<Vec<_>>()
        .join(" ")
}

// A token is either a move number marker ("12.", "12...", or the messy
// "12.Nf3" run-together form) or a move itself.
fn normalize_token(tok: &str) -> String {
    let digit_end = tok
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(tok.len());

    if digit_end > 0 {
        let number = &tok[..digit_end];
        let after = &tok[digit_end..];
        if let Some(rest) = after.strip_prefix('.') {
            let dot_count = 1 + rest.chars().take_while(|&c| c == '.').count();
            let rest = &rest[dot_count - 1..];
            let marker = if dot_count == 1 { "." } else { "..." };
            return if rest.is_empty() {
                format!("{number}{marker}")
            } else {
                format!("{number}{marker} {}", normalize_move(rest))
            };
        }
    }

    normalize_move(tok)
}

fn normalize_move(mv: &str) -> String {
    if let Some(castled) = normalize_castling(mv) {
        return castled;
    }

    let mut chars: Vec<char> = mv.chars().collect();
    if let Some(&first) = chars.first() {
        // N/Q/R/K are unambiguous even lowercase. "b" is left alone because
        // it could mean a bishop move or a b-file pawn move (e.g. "bxc3");
        // telling those apart needs the rest of the game, not just the token.
        if matches!(first, 'n' | 'q' | 'r' | 'k') {
            chars[0] = first.to_ascii_uppercase();
        }
    }

    let mut result: String = chars.into_iter().collect();
    result = result.replace(':', "x");
    if let Some(prefix) = result.strip_suffix("++") {
        result = format!("{prefix}#");
    }
    result
}

fn normalize_castling(mv: &str) -> Option<String> {
    let trailing = mv
        .chars()
        .rev()
        .take_while(|&c| c == '+' || c == '#')
        .count();
    let core_end = mv.len() - trailing;
    let core = &mv[..core_end];
    let suffix = &mv[core_end..];

    let spelled_out = core.to_ascii_uppercase().replace('0', "O");
    match spelled_out.as_str() {
        "O-O" => Some(format!("O-O{suffix}")),
        "O-O-O" => Some(format!("O-O-O{suffix}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_castling_variants() {
        assert_eq!(normalize("1. e4 e5 2. 0-0"), "1. e4 e5 2. O-O\n");
        assert_eq!(normalize("1. e4 e5 2. o-o-o+"), "1. e4 e5 2. O-O-O+\n");
    }

    #[test]
    fn normalizes_capture_colon() {
        assert_eq!(normalize("1. e4 d5 2. N:f3"), "1. e4 d5 2. Nxf3\n");
    }

    #[test]
    fn normalizes_move_number_spacing() {
        assert_eq!(normalize("1.e4 e5 2.Nf3"), "1. e4 e5 2. Nf3\n");
    }

    #[test]
    fn keeps_black_move_ellipsis() {
        assert_eq!(normalize("1. e4 e5 2. Nf3 12..Nf6"), "1. e4 e5 2. Nf3 12... Nf6\n");
    }

    #[test]
    fn capitalizes_unambiguous_piece_letters() {
        assert_eq!(normalize("1. e4 e5 2. nf3 nc6"), "1. e4 e5 2. Nf3 Nc6\n");
    }

    #[test]
    fn normalizes_double_plus_to_hash() {
        assert_eq!(
            normalize("1. e4 e5 2. Qh5 Nc6 3. Qxf7++"),
            "1. e4 e5 2. Qh5 Nc6 3. Qxf7#\n"
        );
    }

    #[test]
    fn preserves_pgn_tag_lines() {
        let input = "[Event \"Test\"]\n1. e4 e5\n";
        assert_eq!(normalize(input), "[Event \"Test\"]\n1. e4 e5\n");
    }
}
