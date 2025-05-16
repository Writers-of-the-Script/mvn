use std::io::LineWriter;
use swc_common::{
    FileName, GLOBALS, Globals, Mark, SourceMap,
    errors::{EmitterWriter, HANDLER, Handler, HandlerFlags},
    sync::Lrc,
};
use swc_ecma_ast::{EsVersion, Program};
use swc_ecma_codegen::{
    Config, Emitter, Node,
    text_writer::{JsWriter, omit_trailing_semi},
};
use swc_ecma_minifier::{
    optimize,
    option::{ExtraOptions, MangleOptions, MinifyOptions},
};
use swc_ecma_parser::{EsSyntax, Syntax, parse_file_as_module};
use swc_ecma_transforms::{
    fixer::{fixer, paren_remover},
    resolver,
};

pub fn minify_js(code: impl AsRef<str>) -> String {
    let dest = LineWriter::new(Vec::new());
    let cm = Lrc::<SourceMap>::default();

    let handler = Handler::with_emitter_and_flags(
        Box::new(EmitterWriter::new(
            Box::new(dest),
            Some(cm.clone()),
            false,
            false,
        )),
        HandlerFlags {
            can_emit_warnings: false,
            treat_err_as_bug: false,
            ..Default::default()
        },
    );

    let program = GLOBALS.set(&Globals::new(), || {
        HANDLER.set(&handler, || {
            let fm = cm.new_source_file(
                FileName::Custom("page.js".into()).into(),
                code.as_ref().into(),
            );

            let unresolved_mark = Mark::new();
            let top_level_mark = Mark::new();

            let program = parse_file_as_module(
                &fm,
                Syntax::Es(EsSyntax::default()),
                EsVersion::default(),
                None,
                &mut Vec::new(),
            )
            .map_err(|err| {
                err.into_diagnostic(&handler).emit();
            })
            .map(Program::Module)
            .map(|module| module.apply(resolver(unresolved_mark, top_level_mark, false)))
            .map(|module| module.apply(paren_remover(None)))
            .unwrap();

            let output = optimize(
                program,
                cm.clone(),
                None,
                None,
                &MinifyOptions {
                    compress: Some(Default::default()),
                    rename: true,
                    wrap: true,
                    enclose: true,
                    mangle: Some(MangleOptions {
                        top_level: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                &ExtraOptions {
                    mangle_name_cache: None,
                    top_level_mark,
                    unresolved_mark,
                },
            );

            let output = output.apply(fixer(None));

            output
        })
    });

    print(cm, &[program], true)
}

fn print<N: Node>(cm: Lrc<SourceMap>, nodes: &[N], minify: bool) -> String {
    let mut buf = Vec::new();

    {
        let mut emitter = Emitter {
            cfg: Config::default().with_minify(minify),
            cm: cm.clone(),
            comments: None,
            wr: omit_trailing_semi(JsWriter::new(cm, "\n", &mut buf, None)),
        };

        for n in nodes {
            n.emit_with(&mut emitter).unwrap();
        }
    }

    String::from_utf8(buf).unwrap()
}
