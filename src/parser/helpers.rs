pub fn translate_operator_symbol(op: &u8) -> String {
    match op {
        b'+' => "plus",
        b'-' => "minus",
        b'*' => "times",
        b'/' => "slash",
        b'.' => "dot",
        b'|' => "pipe",
        b'>' => "greater",
        b'<' => "less",
        b'=' => "equals",
        b'?' => "interrogation",
        b'!' => "exclamation",
        b'~' => "tilde",
        b'%' => "percent",
        b'&' => "ampersand",
        b'#' => "bang",
        b'$' => "dollar",
        b'^' => "power",
        b':' => "colon",
        _ => {
            let val = op.clone() as char;
            panic!("Error! Unexpected operator '{val}' ({op}) to be translated as a function!")
        }
    }
    .to_owned()
}

pub fn generate_operator_function_name(operator_chain: String) -> String {
    format!(
        "__saturnus_operator_{}",
        operator_chain
            .into_bytes()
            .iter()
            .map(translate_operator_symbol)
            .collect::<Vec<String>>()
            .join("_")
    )
}
