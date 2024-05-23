use crossterm::style::StyledContent;

pub fn collapse_str(
    input: StyledContent<String>,
    max_length: usize,
) -> StyledContent<String> {
    let content = input.content().clone();

    let collapsed_text = if content.len() > max_length {
        let collapsed: String = content.chars().take(max_length - 3).collect();

        format!("{}...", collapsed)
    } else {
        content
    };

    let text = if collapsed_text.len() < max_length {
        let padding = max_length - collapsed_text.len();

        format!("{}{}", collapsed_text, " ".repeat(padding))
    } else {
        collapsed_text
    };

    return crossterm::style::StyledContent::new(input.style().clone(), text);
}

pub fn print_table_row(
    cells: Vec<StyledContent<String>>,
    with_underline: bool,
) {
    let column_width = 25;

    let cells_count = cells.len();

    let mut row_length = 0;

    for (index, cell) in cells.into_iter().enumerate() {
        let is_last_cell = index == cells_count - 1;
        let text = collapse_str(cell, column_width);

        row_length += text.content().len();

        let output = if is_last_cell {
            format!("{: <width$}", text, width = column_width)
        } else {
            row_length += 3;
            format!("{: <width$} | ", text, width = column_width)
        };

        print!("{}", output);
    }

    if with_underline {
        println!("\n{}", "-".repeat(row_length));
    } else {
        println!();
    }
}

#[test]
pub fn test_collapse_str() {
    use crossterm::style::Stylize;

    let test_cases = vec![
        (
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, se"
                .to_owned(),
            20,
            "Lorem ipsum dolor...".to_owned(),
        ),
        (
            "Lorem ipsum dolor".to_owned(),
            20,
            "Lorem ipsum dolor   ".to_owned(),
        ),
        (
            "Lorem ipsum dolor si".to_owned(),
            20,
            "Lorem ipsum dolor si".to_owned(),
        ),
        ("".to_owned(), 20, "                    ".to_owned()),
    ];

    for (input, max_length, expected) in test_cases {
        let actual = collapse_str(input.white(), max_length);

        assert_eq!(actual, expected.white())
    }
}
