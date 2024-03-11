use crate::manifest::lexer::Span;
use annotate_snippets::{Annotation, AnnotationType, Renderer, Slice, Snippet, SourceAnnotation};
use sbor::rust::cmp::min;

pub fn create_snippet(s: &str, span: &Span, title: &str, label: &str) -> String {
    let lines_cnt = s.lines().count();
    let mut span = span.clone();

    // Surround span with few lines for more context
    if span.start.line_number > 5 {
        span.start.line_number -= 5;
    } else {
        span.start.line_number = 1;
    }
    span.end.line_number = min(span.end.line_number + 5, lines_cnt);

    let mut source = String::new();
    let mut skipped_chars = 0;
    for (i, line) in s.lines().enumerate() {
        if (i + 1) < span.start.line_number {
            // Add 1 for '\n' character
            skipped_chars += line.chars().count() + 1;
        } else if (i + 1) <= span.end.line_number {
            source.push_str(line.into());
            source.push('\n');
        } else if (i + 1) > span.end.line_number {
            break;
        }
    }

    span.start.full_index -= skipped_chars;
    span.end.full_index -= skipped_chars;

    let snippet = Snippet {
        slices: vec![Slice {
            source: source.as_str(),
            line_start: span.start.line_number,
            origin: None,
            fold: false,
            annotations: vec![SourceAnnotation {
                label: label,
                annotation_type: AnnotationType::Info,
                range: (span.start.full_index, span.end.full_index),
            }],
        }],
        title: Some(Annotation {
            label: Some(title),
            id: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
    };

    let renderer = Renderer::styled();
    let s = renderer.render(snippet).to_string();
    s
}
