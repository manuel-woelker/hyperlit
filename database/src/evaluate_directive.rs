use crate::Database;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_model::directives::parse_directive;

pub fn evaluate_directive<'a>(
    string: &str,
    database: &'a dyn Database,
) -> HyperlitResult<DirectiveEvaluation<'a>> {
    let string = string.trim();
    let prefix = "§{";
    Ok(if string.starts_with(prefix) && string.ends_with("}") {
        let directive = parse_directive(string)?;
        match directive {
            hyperlit_model::directives::Directive::IncludeByTag { tag } => {
                let segments = database.get_segments_by_tag(&tag)?;
                DirectiveEvaluation::Segments { segments }
            }
            hyperlit_model::directives::Directive::IncludeRest => {
                let segments = database.get_all_segments()?;
                let rest_segments = segments.into_iter().filter(|s| !s.is_included).collect();
                DirectiveEvaluation::Segments {
                    segments: rest_segments,
                }
            }
        }
    } else {
        DirectiveEvaluation::NoDirective
    })
}

#[cfg(test)]
mod tests {
    use crate::Database;
    use crate::evaluate_directive::{DirectiveEvaluation, evaluate_directive};
    use crate::in_memory_database::InMemoryDatabase;
    use hyperlit_base::result::HyperlitResult;
    use hyperlit_model::location::Location;
    use hyperlit_model::segment::Segment;

    fn make_segments(how_many: usize) -> Vec<Segment> {
        vec![Segment::new(0, "title", vec![], "text", Location::new("path", 1)); how_many]
    }

    #[test]
    fn test_evaluate_include_rest_directive() -> HyperlitResult<()> {
        let mut database = InMemoryDatabase::new();
        database.add_segments(make_segments(3))?;
        database.set_segment_included(1)?;
        let evaluation = evaluate_directive("§{@include_rest}", &database)?;
        match evaluation {
            DirectiveEvaluation::Segments { segments } => {
                assert_eq!(
                    segments.iter().map(|s| s.id).collect::<Vec<_>>(),
                    vec![0, 2]
                );
            }
            _ => panic!("should be segments, instead got: {evaluation:?}"),
        }
        Ok(())
    }

    #[test]
    fn test_evaluate_include_by_tag() -> HyperlitResult<()> {
        let mut database = InMemoryDatabase::new();
        database.add_segments(make_segments(3))?;
        database.add_segments(vec![Segment::new(
            0,
            "title A",
            vec!["the_tag".to_string()],
            "text",
            Location::new("path", 1),
        )])?;
        database.add_segments(make_segments(3))?;
        database.add_segments(vec![Segment::new(
            0,
            "title B",
            vec!["the_tag".to_string()],
            "text",
            Location::new("path", 1),
        )])?;
        let evaluation = evaluate_directive("§{@include_by_tag:#the_tag}", &database)?;
        match evaluation {
            DirectiveEvaluation::Segments { segments } => {
                assert_eq!(
                    segments.iter().map(|s| s.id).collect::<Vec<_>>(),
                    vec![3, 7]
                );
            }
            _ => panic!("should be segments, instead got: {evaluation:?}"),
        }
        Ok(())
    }

    #[test]
    fn test_evaluate_no_directive() -> HyperlitResult<()> {
        fn evaluate(string: &str) -> HyperlitResult<()> {
            let database = InMemoryDatabase::new();
            let evaluation = evaluate_directive(string, &database)?;
            match evaluation {
                DirectiveEvaluation::NoDirective => {
                    // Ok
                }
                _ => panic!("should be no directive, instead got: {evaluation:?}"),
            }
            Ok(())
        }
        evaluate("")?;
        evaluate("    ")?;
        evaluate("§{    }x")?;
        evaluate("§{    x")?;
        evaluate("}")?;
        evaluate("§ {  }")?;
        Ok(())
    }
}
