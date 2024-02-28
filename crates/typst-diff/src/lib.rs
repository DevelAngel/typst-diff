mod content;

use std::collections::HashSet;

use comemo::{Prehashed, Track, Tracked, Validate};
use ecow::EcoVec;

use typst::diag::{warning, SourceDiagnostic, SourceResult};
use typst::engine::{Engine, Route};
use typst::eval::{self, Tracer};
use typst::foundations::{Content, Smart, StyleChain};
use typst::introspection::{Introspector, Locator};
use typst::layout::{Abs, LayoutRoot};
use typst::model::Document;
use typst::syntax::Span;
use typst::text::{HighlightElem, SpaceElem, StrikeElem};
use typst::util;
use typst::visualize::{Color, Paint, Stroke};
use typst::World;

use crate::content::DiffableContent;

use similar::{capture_diff_slices, Algorithm, ChangeTag};

/// Compile a source file into a fully layouted document.
///
/// - Returns `Ok(document)` if there were no fatal errors.
/// - Returns `Err(errors)` if there were fatal errors.
///
/// Requires a mutable reference to a tracer. Such a tracer can be created with
/// `Tracer::new()`. Independently of whether compilation succeeded, calling
/// `tracer.warnings()` after compilation will return all compiler warnings.
#[tracing::instrument(skip_all)]
pub fn compile_diff(
    world_one: &dyn World,
    world_two: &dyn World,
    tracer: &mut Tracer,
) -> SourceResult<Document> {
    // Call `track` on the world just once to keep comemo's ID stable.
    let world_one = world_one.track();
    let world_two = world_two.track();

    // Try to evaluate the source file into a module.
    let module_one = eval::eval(
        world_one,
        Route::default().track(),
        tracer.track_mut(),
        &world_one.main(),
    )
    .map_err(deduplicate)?;
    let module_two = eval::eval(
        world_two,
        Route::default().track(),
        tracer.track_mut(),
        &world_two.main(),
    )
    .map_err(deduplicate)?;

    let content_one = module_one.content();
    let content_two = module_two.content();
    let content_merged = diff_content(content_one, content_two);

    // Typeset the module's content, relayouting until convergence.
    typeset(world_one, tracer, &content_merged).map_err(deduplicate)
}

/// Relayout until introspection converges.
fn typeset(
    world: Tracked<dyn World + '_>,
    tracer: &mut Tracer,
    content: &Content,
) -> SourceResult<Document> {
    let library = world.library();
    let styles = StyleChain::new(&library.styles);

    let mut iter = 0;
    let mut document = Document::default();

    // Relayout until all introspections stabilize.
    // If that doesn't happen within five attempts, we give up.
    loop {
        tracing::info!("Layout iteration {iter}");

        // Clear delayed errors.
        tracer.delayed();

        let constraint = <Introspector as Validate>::Constraint::new();
        let mut locator = Locator::new();
        let mut engine = Engine {
            world,
            route: Route::default(),
            tracer: tracer.track_mut(),
            locator: &mut locator,
            introspector: document.introspector.track_with(&constraint),
        };

        // Layout!
        document = content.layout_root(&mut engine, styles)?;
        document.introspector.rebuild(&document.pages);
        iter += 1;

        if document.introspector.validate(&constraint) {
            break;
        }

        if iter >= 5 {
            tracer.warn(warning!(
                Span::detached(), "layout did not converge within 5 attempts";
                hint: "check if any states or queries are updating themselves"
            ));
            break;
        }
    }

    // Promote delayed errors.
    let delayed = tracer.delayed();
    if !delayed.is_empty() {
        return Err(delayed);
    }

    Ok(document)
}

/// Deduplicate diagnostics.
fn deduplicate(mut diags: EcoVec<SourceDiagnostic>) -> EcoVec<SourceDiagnostic> {
    let mut unique = HashSet::new();
    diags.retain(|diag| {
        let hash = util::hash128(&(&diag.span, &diag.message));
        unique.insert(hash)
    });
    diags
}

fn diff_content(content_one: Content, content_two: Content) -> Content {
    fn fold_content<'a>(
        (mut vec, append): (Vec<DiffableContent<'a>>, bool),
        content: &'a Prehashed<Content>,
    ) -> (Vec<DiffableContent<'a>>, bool) {
        if content.to::<SpaceElem>().is_some() {
            let last = vec.pop().expect("first element is never a SpaceElem");
            let last = last.append(content);
            vec.push(last);
            (vec, true)
        } else if append {
            let last = vec.pop().unwrap();
            let last = last.append(content);
            vec.push(last);
            (vec, false)
        } else {
            vec.push(content.into());
            (vec, false)
        }
    }

    let seq_one: Vec<DiffableContent> = match content_one.to_sequence() {
        Some(seq) => {
            seq.fold((Vec::<DiffableContent>::new(), false), fold_content)
                .0
        }
        None => vec![DiffableContent::from(&content_one)],
    };
    let seq_two: Vec<DiffableContent> = match content_two.to_sequence() {
        Some(seq) => {
            seq.fold((Vec::<DiffableContent>::new(), false), fold_content)
                .0
        }
        None => vec![DiffableContent::from(&content_two)],
    };
    tracing::trace!("seq_one:\n{seq_one:#?}");
    tracing::trace!("seq_two:\n{seq_two:#?}");

    let ops = capture_diff_slices(Algorithm::Lcs, &seq_one, &seq_two);
    let changes: Vec<_> = ops
        .iter()
        .flat_map(|x| x.iter_slices(&seq_one, &seq_two))
        .collect();
    let content = Vec::with_capacity(changes.len());
    let content = changes.iter().fold(content, create_diff_content);
    let content = Content::sequence(content);
    tracing::trace!("merged content:\n{content:#?}");
    content
}

fn create_diff_content(
    mut content: Vec<Content>,
    (tag, next): &(ChangeTag, &[DiffableContent]),
) -> Vec<Content> {
    let body: Content = Content::sequence(
        next.iter()
            .flat_map(|x| x.as_slice().iter().map(|x| (*x).clone())),
    );
    match tag {
        ChangeTag::Equal => {
            content.push(body);
        }
        ChangeTag::Delete => {
            let strike_elem = StrikeElem::new(body).with_stroke(Smart::Custom(Stroke {
                paint: Smart::Auto,
                thickness: Smart::Custom(Abs::pt(1.5).into()),
                line_cap: Smart::Auto,
                line_join: Smart::Auto,
                dash_pattern: Smart::Auto,
                miter_limit: Smart::Auto,
            }));
            let strike_elem = Content::new(strike_elem);
            let highlight_elem =
                HighlightElem::new(strike_elem).with_fill(Paint::Solid(Color::RED));
            let highlight_elem = Content::new(highlight_elem);
            content.push(highlight_elem);
        }
        ChangeTag::Insert => {
            let highlight_elem = HighlightElem::new(body).with_fill(Paint::Solid(Color::GREEN));
            let highlight_elem = Content::new(highlight_elem);
            content.push(highlight_elem);
        }
    }
    content
}
