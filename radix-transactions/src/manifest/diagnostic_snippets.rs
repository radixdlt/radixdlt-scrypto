use crate::manifest::compiler::CompileErrorDiagnosticsStyle;
use crate::manifest::token::Span;
use annotate_snippets::{Annotation, AnnotationType, Renderer, Slice, Snippet, SourceAnnotation};
use sbor::rust::cmp::min;

pub fn create_snippet(
    s: &str,
    span: &Span,
    title: &str,
    label: &str,
    style: CompileErrorDiagnosticsStyle,
) -> String {
    let lines_cnt = s.lines().count();

    // Surround span with few lines for more context
    let line_start = if span.start.line_number() > 5 {
        span.start.line_number() - 5
    } else {
        1
    };
    let line_end = min(span.end.line_number() + 5, lines_cnt);

    let mut source = String::new();
    let mut skipped_chars = 0;
    for (i, line) in s.lines().enumerate() {
        if (i + 1) < line_start {
            // Add 1 for '\n' character
            skipped_chars += line.chars().count() + 1;
        } else if (i + 1) <= line_end {
            source.push_str(line.into());
            source.push('\n');
        } else if (i + 1) > line_end {
            break;
        }
    }

    // Normalize spans indicating end of source
    let mut annotation_start_index = min(span.start.full_index, s.len());
    let mut annotation_end_index = min(span.end.full_index, s.len());

    if annotation_start_index == annotation_end_index {
        // Add 1 to let the ^ be displayed to indicate end of source
        annotation_end_index += 1;
    }

    annotation_start_index -= skipped_chars;
    annotation_end_index -= skipped_chars;

    let snippet = Snippet {
        slices: vec![Slice {
            source: source.as_str(),
            line_start: line_start,
            origin: None,
            fold: false,
            annotations: vec![SourceAnnotation {
                label: label,
                annotation_type: AnnotationType::Error,
                // Range require unicode char indices, which matches indices used in Span
                range: (annotation_start_index, annotation_end_index),
            }],
        }],
        title: Some(Annotation {
            label: Some(title),
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
    };

    let renderer = match style {
        CompileErrorDiagnosticsStyle::PlainText => Renderer::plain(),
        CompileErrorDiagnosticsStyle::TextTerminalColors => Renderer::styled(),
    };

    let s = renderer.render(snippet).to_string();
    s
}
