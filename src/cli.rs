use std::cmp;

pub enum Commands {
    Scan,
    Clean,
    List,
    Help,
    Exit
}

pub fn find_suggestion(input: &str) -> Option<&'static str> {
    let commands = ["scan", "clean", "list", "help"];
    
    commands.iter()
        .map(|&cmd| (levenshtein(input, cmd), cmd))
        .filter(|(dist, _)| *dist < 3) 
        .min_by_key(|(dist, _)| *dist)
        .map(|(_, cmd)| cmd)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b_len = b.chars().count();
    if a.is_empty() { return b_len; }
    
    let mut column: [usize; 16] = [0; 16]; 
    for i in 1..=b_len { column[i] = i; }

    for (i, char_a) in a.chars().enumerate() {
        let mut last_diagonal = i;
        column[0] = i + 1;
        for (j, char_b) in b.chars().enumerate() {
            let old_diagonal = column[j + 1];
            let cost = if char_a == char_b { 0 } else { 1 };
            column[j + 1] = cmp::min(
                cmp::min(column[j + 1] + 1, column[j] + 1),
                last_diagonal + cost
            );
            last_diagonal = old_diagonal;
        }
    }
    column[b_len]
}