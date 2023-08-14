use crate::output::Output;

pub fn human(outputs: &[Output]) -> String {
    outputs
        .iter()
        .filter(|output| output.file_name().is_some())
        .filter_map(crate::Output::rendered)
        .cloned()
        .collect()
}

pub fn github_summary(outputs: &[Output]) -> String {
    let success = outputs.iter().any(crate::Output::success);

    if !outputs.is_empty() {
        println!("| Type | Message |");
        println!("| ---- | ------- |");

        outputs
            .iter()
            .filter(|output| output.file_name().is_some())
            .map(|output| {
                format!(
                    "| {} | {} |",
                    output.level().unwrap_or_else(|| "Unknow".to_owned()),
                    output
                        .message()
                        .unwrap_or_else(|| "No message".to_owned())
                        .lines()
                        .take(1)
                        .collect::<String>(),
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else if success {
        "\u{1f980} Cargo is Happy !".to_owned()
    } else {
        "\u{1f612} Cargo is Sad !".to_owned()
    }
}

pub fn github_pr_annotation(outputs: &[Output]) -> String {
    outputs
        .iter()
        .filter(|output| output.file_name().is_some())
        .map(|output| {
            let opts = vec![
                output
                    .file_name()
                    .map(|file_name| format!("file={file_name}")),
                output.line_start().map(|line| format!("line={line}")),
                output
                    .line_end()
                    .map(|end_line| format!("endLine={end_line}")),
                output
                    .column_start()
                    .map(|col_start| format!("col={col_start}")),
                output
                    .column_end()
                    .map(|col_end| format!("endColumn={col_end}")),
                output.message().map(|title| format!("title={title}")),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(",");

            format!(
                "::{} {opts}::{}",
                output.level().unwrap_or_else(|| "notice".to_owned()),
                output
                    .rendered()
                    .and_then(|rendered| Some(rendered.as_str()))
                    .unwrap_or_else(|| "No message")
                    .replace('\n', "%0A"),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
