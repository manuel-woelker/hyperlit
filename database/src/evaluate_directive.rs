use crate::Database;
use hyperlit_base::result::HyperlitResult;
use hyperlit_model::directive_evaluation::DirectiveEvaluation;
use hyperlit_model::directives::parse_directive;

pub fn evaluate_directive<'a>(
    directive_string: &str,
    database: &'a dyn Database,
) -> HyperlitResult<DirectiveEvaluation<'a>> {
    let directive = parse_directive(directive_string)?;
    Ok(match directive {
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
        vec![Segment::new(0, "title", vec![], "text", Location::new("path", 1, 2)); how_many]
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
            Location::new("path", 1, 2),
        )])?;
        database.add_segments(make_segments(3))?;
        database.add_segments(vec![Segment::new(
            0,
            "title B",
            vec!["the_tag".to_string()],
            "text",
            Location::new("path", 1, 2),
        )])?;
        let evaluation = evaluate_directive("§{@include_by_tag:#the_tag}", &database)?;
        match evaluation {
            DirectiveEvaluation::Segments { segments } => {
                assert_eq!(
                    segments.iter().map(|s| s.id).collect::<Vec<_>>(),
                    vec![3, 7]
                );
            }
        }
        Ok(())
    }
}
