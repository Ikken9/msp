pub fn print_matrix<T: ToString + PartialEq>(matrix: &[Vec<T>], placeholder: Option<T>, placeholder_text: &str) {
    let mut max_width = placeholder_text.len();
    for row in matrix {
        for value in row {
            let width = value.to_string().len();
            if width > max_width {
                max_width = width;
            }
        }
    }

    for row in matrix {
        for value in row {
            if let Some(placeholder) = &placeholder {
                if value == placeholder {
                    print!("{:^width$} ", placeholder_text, width = max_width);
                    continue;
                }
            }
            print!("{:^width$} ", value.to_string(), width = max_width);
        }
        println!();
    }
}

pub fn print_matrix_with_labels(matrix: &[Vec<usize>], node_ids: &[String], placeholder: &str) {
    let mut max_width = placeholder.len();
    for row in matrix {
        for &value in row {
            if value != usize::MAX {
                let width = value.to_string().len();
                if width > max_width {
                    max_width = width;
                }
            }
        }
    }
    for node_id in node_ids {
        let width = node_id.len();
        if width > max_width {
            max_width = width;
        }
    }

    print!("{:>width$} ", "", width = max_width);
    for node_id in node_ids {
        print!("{:>width$} ", node_id, width = max_width);
    }
    println!();

    for (i, row) in matrix.iter().enumerate() {
        print!("{:>width$} ", node_ids[i], width = max_width);
        for &value in row {
            if value == usize::MAX {
                print!("{:>width$} ", placeholder, width = max_width);
            } else {
                print!("{:>width$} ", value, width = max_width);
            }
        }
        println!();
    }
}

