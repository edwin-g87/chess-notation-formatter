# chessfmt

Chess movetext shows up in a lot of inconsistent styles depending on where it
came from: old books use `:` for a capture instead of `x`, some software
writes castling as `0-0` instead of `O-O`, mate used to be written `++`
instead of `#`, and move numbers get typed as `1.e4` or `1 . e4` or `1. e4`
depending on who typed them. None of these are wrong exactly, they're just
different conventions mixed together, and it makes the text annoying to
diff, grep, or paste into something that expects one consistent style.

`chessfmt` reads movetext and rewrites it in a single consistent style. It
leaves PGN tag lines (the `[Event "..."]` header lines) alone and only
touches the moves themselves.

## Usage

From a file:

```
cargo run -- game.txt
```

From stdin:

```
cat game.txt | cargo run
```

or explicitly with `-`:

```
cargo run -- -
```

## Example

Input:

```
[Event "Casual game"]
1.e4 e5 2.Nf3 nc6 3.Bb5 a6 4.0-0 Nf6 5.Qxf7++
```

Output:

```
[Event "Casual game"]
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. O-O Nf6 5. Qxf7#
```

## What it normalizes right now

- Move numbers: `1.e4` and `1 .e4` style spacing collapses to `1. e4`; a
  standalone black move keeps its `...` marker (`12...Nf6`).
- Castling: `0-0`, `o-o`, `0-0-0` all become `O-O` / `O-O-O`.
- Captures: `N:f3` becomes `Nxf3`.
- Mate: `Qxf7++` becomes `Qxf7#`.
- Piece letters: `nf3` becomes `Nf3`. This applies to N, Q, R, K. Lowercase
  `b` is left alone on purpose, since `bxc3` is genuinely ambiguous between
  a bishop capture and a b-file pawn capture without replaying the game.

Lines starting with `[` (PGN tag pairs) pass through unchanged.

## Known gaps

This is an early version. Not handled yet:

- Pawn promotion notation (`e8=Q`, `e8/Q`, `e8Q`)
- Comments (`{...}`) and numeric annotation glyphs (`$1`, `!?`, etc.)
- Disambiguating the lowercase `b` case by tracking board state
- Validating that the moves are actually legal, as opposed to just
  reformatting the text

## License

MIT, see [LICENSE](LICENSE).
