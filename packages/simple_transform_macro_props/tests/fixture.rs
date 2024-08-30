use std::path::PathBuf;

use swc_ecma_parser::{Syntax, TsSyntax};
use swc_ecma_transforms_testing::test_fixture;
use swc_plugin_simple_transform_macro_props::{
    simple_transform_macro_props, SimpleTransformPluginOptions,
};

fn ts_syntax() -> Syntax {
    Syntax::Typescript(TsSyntax {
        tsx: true,
        ..Default::default()
    })
}

#[testing::fixture("tests/fixture/**/input.tsx")]
fn simple_transform_macro_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.tsx");
    test_fixture(
        ts_syntax(),
        &|_t| {
            simple_transform_macro_props(SimpleTransformPluginOptions {
                packages: vec!["react-emotion".into()],
            })
        },
        &input,
        &output,
        Default::default(),
    )
}
