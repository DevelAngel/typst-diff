use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Timelike};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::{self, termcolor};
use ecow::{eco_format, EcoString};
use parking_lot::RwLock;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use termcolor::{ColorChoice, StandardStream};
use typst::diag::{bail, At, Severity, SourceDiagnostic, StrResult};
use typst::eval::Tracer;
use typst::foundations::Datetime;
use typst::layout::Frame;
use typst::model::Document;
use typst::syntax::{FileId, Source, Span};
use typst::visualize::Color;
use typst::{World, WorldExt};

use crate::args::{CompileDiffCommand, DiagnosticFormat, OutputFormat};
use crate::timings::Timer;
use crate::watch::Status;
use crate::world::SystemWorld;
use crate::{color_stream, set_failed};

/// Execute a compilation command.
pub fn compile(mut timer: Timer, mut command: CompileDiffCommand) -> StrResult<()> {
    let mut world = SystemWorld::new(&command.common)?;
    timer.record(&mut world, |world| compile_once(world, &mut command))??;
    Ok(())
}

/// Compile a single time.
///
/// Returns whether it compiled without errors.
#[typst_macros::time(name = "compile once")]
fn compile_once(
    world: &mut SystemWorld,
    command: &mut CompileDiffCommand,
) -> StrResult<()> {
    let start = std::time::Instant::now();

    // Check if main file can be read and opened.
    if let Err(errors) = world.source(world.main()).at(Span::detached()) {
        set_failed();

        print_diagnostics(world, &errors, &[], command.common.diagnostic_format)
            .map_err(|err| eco_format!("failed to print diagnostics ({err})"))?;

        return Ok(());
    }

    let mut tracer = Tracer::new();
    let result = typst::compile(world, &mut tracer);
    let warnings = tracer.warnings();

    match result {
        // Export the PDF / PNG.
        Ok(document) => {
            export(world, &document, command, watching)?;
            let duration = start.elapsed();

            print_diagnostics(world, &[], &warnings, command.common.diagnostic_format)
                .map_err(|err| eco_format!("failed to print diagnostics ({err})"))?;

            if let Some(open) = command.open.take() {
                open_file(open.as_deref(), &command.output())?;
            }
        }

        // Print diagnostics.
        Err(errors) => {
            set_failed();

            print_diagnostics(
                world,
                &errors,
                &warnings,
                command.common.diagnostic_format,
            )
            .map_err(|err| eco_format!("failed to print diagnostics ({err})"))?;
        }
    }

    Ok(())
}
