use std::{collections::HashMap, path::PathBuf};

use swc_ecma_parser::{Syntax, TsConfig};
use swc_ecma_transforms_testing::test_fixture;
use swc_ecma_visit::as_folder;
use swc_plugin_transform_import::{transform_import, TransformImportConfig};

fn ts_syntax() -> Syntax {
    Syntax::Typescript(TsConfig {
        tsx: true,
        ..Default::default()
    })
}

#[testing::fixture("tests/fixture/**/input.tsx")]
fn transform_import_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.ts");

    test_fixture(
        ts_syntax(),
        &|_t| {
            as_folder(transform_import(HashMap::from([
                (
                    "antd".to_string(),
                    TransformImportConfig {
                        transform_case: "kebab_case".to_string(),
                        transform: "antd/lib/[[member]]".to_string(),
                        style_path: Some("antd/lib/[[member]]/style".to_string()),
                        keep_import_conversion: false,
                    },
                ),
                (
                    "lodash".to_string(),
                    TransformImportConfig {
                        transform_case: "".to_string(),
                        transform: "lodash/[[member]]".to_string(),
                        style_path: None,
                        keep_import_conversion: false,
                    },
                ),
            ])))
        },
        &input,
        &output,
        Default::default(),
    )
}
