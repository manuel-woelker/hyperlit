use crate::segment::Segment;

#[derive(Debug)]
pub enum DirectiveEvaluation<'a> {
    Segments { segments: Vec<&'a Segment> },
    NoDirective,
}
