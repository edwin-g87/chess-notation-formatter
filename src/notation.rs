//! Normalizes messy chess movetext into a consistent style:
//! move numbers followed by ". " (or "... " for a black move shown on its
//! own), uppercase piece letters, "x" for captures instead of ":", "#"
//! instead of "++" for mate, and "=Q" for promotion regardless of whether
//! the source used "=", "/", or nothing at all. Lines that look like PGN
//! tag pairs (starting with '[') are passed through untouched.

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
    normalize_promotion(&result)
}

// Promotion shows up as "e8=Q", "e8/Q", or the bare "e8Q", with the piece
// letter in either case. All of these mean the same thing, so they all
// collapse to the "=Q" form. Check/mate markers after the promoted piece
// (already reduced to a single "+" or "#" by the time this runs) are kept.
fn normalize_promotion(mv: &str) -> String {
    let trailing = mv.chars().rev().take_while(|&c| c == '+' || c == '#').count();
    let core_end = mv.len() - trailing;
    let core = &mv[..core_end];
    let suffix = &mv[core_end..];

    let mut chars: Vec<char> = core.chars().collect();
    let Some(&last) = chars.last() else {
        return mv.to_string();
    };
    let piece = match last.to_ascii_uppercase() {
        p @ ('Q' | 'R' | 'B' | 'N') => p,
        _ => return mv.to_string(),
    };
    chars.pop();
    if matches!(chars.last(), Some('=') | Some('/')) {
        chars.pop();
    }
    let square: String = chars.into_iter().collect();
    if is_promotion_square(&square) {
        format!("{square}={piece}{suffix}")
    } else {
        mv.to_string()
    }
}

// A promotion destination is a pawn move that lands on the back rank:
// the token ends in a file letter followed by "1" or "8". The characters
// before that (plain "e8" or a capture like "exd8") don't matter here.
fn is_promotion_square(square: &str) -> bool {
    let chars: Vec<char> = square.chars().collect();
    if chars.len() < 2 {
        return false;
    }
    let rank = chars[chars.len() - 1];
    let file = chars[chars.len() - 2];
    matches!(rank, '1' | '8') && matches!(file, 'a'..='h')
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
    fn normalizes_promotion_variants() {
        assert_eq!(normalize("39. b8=Q"), "39. b8=Q\n");
        assert_eq!(normalize("39. b8Q"), "39. b8=Q\n");
        assert_eq!(normalize("39. b8/Q"), "39. b8=Q\n");
        assert_eq!(normalize("39. b8=q"), "39. b8=Q\n");
    }

    #[test]
    fn normalizes_promotion_with_capture_and_check() {
        assert_eq!(normalize("39. exd8=N+"), "39. exd8=N+\n");
        assert_eq!(normalize("39. exd8N#"), "39. exd8=N#\n");
    }

    #[test]
    fn leaves_non_promotion_moves_alone() {
        assert_eq!(normalize("1. e4 e5 2. Nf3 Nc6"), "1. e4 e5 2. Nf3 Nc6\n");
    }

    #[test]
    fn preserves_pgn_tag_lines() {
        let input = "[Event \"Test\"]\n1. e4 e5\n";
        assert_eq!(normalize(input), "[Event \"Test\"]\n1. e4 e5\n");
    }
}
