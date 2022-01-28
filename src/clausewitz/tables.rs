const fn string_litteral_content_table() -> [bool; 256] {
    let mut table = [true; 256];
    table[b'"' as usize] = false;

    table
}
const fn identifier_table() -> [bool; 256] {
    let mut table = [false; 256];
    table[b'a' as usize] = true;
    table[b'b' as usize] = true;
    table[b'c' as usize] = true;
    table[b'd' as usize] = true;
    table[b'e' as usize] = true;
    table[b'f' as usize] = true;
    table[b'g' as usize] = true;
    table[b'h' as usize] = true;
    table[b'i' as usize] = true;
    table[b'j' as usize] = true;
    table[b'k' as usize] = true;
    table[b'l' as usize] = true;
    table[b'm' as usize] = true;
    table[b'n' as usize] = true;
    table[b'o' as usize] = true;
    table[b'p' as usize] = true;
    table[b'q' as usize] = true;
    table[b'r' as usize] = true;
    table[b's' as usize] = true;
    table[b't' as usize] = true;
    table[b'u' as usize] = true;
    table[b'v' as usize] = true;
    table[b'w' as usize] = true;
    table[b'x' as usize] = true;
    table[b'y' as usize] = true;
    table[b'z' as usize] = true;
    table[b'A' as usize] = true;
    table[b'B' as usize] = true;
    table[b'C' as usize] = true;
    table[b'D' as usize] = true;
    table[b'E' as usize] = true;
    table[b'F' as usize] = true;
    table[b'G' as usize] = true;
    table[b'H' as usize] = true;
    table[b'I' as usize] = true;
    table[b'J' as usize] = true;
    table[b'K' as usize] = true;
    table[b'L' as usize] = true;
    table[b'M' as usize] = true;
    table[b'N' as usize] = true;
    table[b'O' as usize] = true;
    table[b'P' as usize] = true;
    table[b'Q' as usize] = true;
    table[b'R' as usize] = true;
    table[b'S' as usize] = true;
    table[b'T' as usize] = true;
    table[b'U' as usize] = true;
    table[b'V' as usize] = true;
    table[b'W' as usize] = true;
    table[b'X' as usize] = true;
    table[b'Y' as usize] = true;
    table[b'Z' as usize] = true;
    table[b'_' as usize] = true;
    table[b'.' as usize] = true;
    table[b'0' as usize] = true;
    table[b'1' as usize] = true;
    table[b'2' as usize] = true;
    table[b'3' as usize] = true;
    table[b'4' as usize] = true;
    table[b'5' as usize] = true;
    table[b'6' as usize] = true;
    table[b'7' as usize] = true;
    table[b'8' as usize] = true;
    table[b'9' as usize] = true;
    table[b':' as usize] = true;
    table
}

const fn token_table() -> [bool; 256] {
    let mut table = [false; 256];
    table[b'=' as usize] = true;
    table[b'{' as usize] = true;
    table[b'}' as usize] = true;
    table
}

pub fn is_string_litteral_contents(char: char) -> bool {
    string_litteral_content_table()[char as usize]
}
pub fn is_identifier_char(char: char) -> bool {
    identifier_table()[char as usize]
}

const fn space_table() -> [bool; 256] {
    let mut table = [false; 256];
    table[b' ' as usize] = true;
    table[b'\n' as usize] = true;
    table[b'\t' as usize] = true;
    table[b'\r' as usize] = true;
    table
}

pub fn is_space(c: char) -> bool {
    space_table()[c as usize]
}

pub fn is_digit(char: char) -> bool {
    char.is_digit(10)
}

pub fn is_token(char: char) -> bool {
    token_table()[char as usize]
}
