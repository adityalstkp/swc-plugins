use core::panic;
use std::collections::HashMap;

use convert_case::Casing;
use serde::Deserialize;
use swc_ecma_ast::{
    ImportDecl, ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier, ModuleDecl,
    ModuleExportName, ModuleItem, Program,
};
use swc_ecma_visit::{noop_visit_mut_type, VisitMut, VisitMutWith};
use swc_plugin_macro::plugin_transform;
use swc_plugin_proxy::TransformPluginProgramMetadata;
use tracing::debug;

#[plugin_transform]
fn transform_import_plugin(mut program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config_str = &data.get_transform_plugin_config().expect("invalid config");
    let configs = serde_json::from_str(config_str).expect("cannot parse configs");
    program.visit_mut_with(&mut transform_import(configs));
    program
}

pub fn transform_import(configs: TransformImportConfigs) -> impl VisitMut {
    TransformImport { configs }
}

enum TransformCase {
    SnakeCase(String), // snake_case
    KebabCase(String), // kebab-case
    NonTransformCase(String),
}

impl TransformCase {
    fn convert_case(&self) -> String {
        match self {
            TransformCase::SnakeCase(s) => s.to_case(convert_case::Case::Snake),
            TransformCase::KebabCase(s) => s.to_case(convert_case::Case::Kebab),
            TransformCase::NonTransformCase(s) => s.to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformImportConfig {
    pub transform: String,
    pub style_path: Option<String>,
    pub transform_case: String,
    #[serde(default)]
    pub keep_import_specifier: bool,
}

pub type TransformImportConfigs = HashMap<String, TransformImportConfig>;

struct TransformImport {
    configs: TransformImportConfigs,
}

impl TransformImport {
    fn convert_member_case(&self, c: &String, val: &String) -> String {
        let case = match c.as_str() {
            "snake_case" => TransformCase::SnakeCase(val.to_string()),
            "kebab_case" => TransformCase::KebabCase(val.to_string()),
            _ => TransformCase::NonTransformCase(val.to_string()),
        };
        case.convert_case()
    }

    fn transform_import_path(
        &self,
        transform: &String,
        member: &String,
        src: &swc_ecma_ast::Str,
    ) -> swc_ecma_ast::Str {
        let transformed_path = transform.replace("[[member]]", &member);
        debug!("transformed_path: {}", transformed_path);

        swc_ecma_ast::Str {
            span: src.span,
            value: transformed_path.into(),
            raw: None,
        }
    }
}

impl VisitMut for TransformImport {
    // to reduce bin size
    noop_visit_mut_type!();

    fn visit_mut_module_items(&mut self, nodes: &mut Vec<ModuleItem>) {
        let mut transformed_nodes: Vec<ModuleItem> = vec![];
        for (_, node) in nodes.iter().enumerate() {
            match node {
                ModuleItem::ModuleDecl(n) => match n {
                    ModuleDecl::Import(import_dclr) => {
                        let import_src = &import_dclr.src.value.to_string();

                        if let Some(config) = self.configs.get(import_src) {
                            debug!("gottem import src pkg: {}", import_src);

                            let is_default_import_exist = import_dclr.specifiers.iter().any(|s| {
                                if let ImportSpecifier::Default(_) | ImportSpecifier::Namespace(_) =
                                    s
                                {
                                    true
                                } else {
                                    false
                                }
                            });

                            if is_default_import_exist {
                                panic!(
                                    "[NOTICE] you are importing all module for {}, please don't",
                                    import_src
                                )
                            }

                            // if type only, push it and leave it
                            if import_dclr.type_only {
                                let new_node =
                                    ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                                        span: import_dclr.span,
                                        phase: Default::default(),
                                        specifiers: import_dclr.specifiers.clone(),
                                        type_only: import_dclr.type_only,
                                        src: import_dclr.src.clone(),
                                        with: import_dclr.with.clone(),
                                    }));
                                transformed_nodes.push(new_node);
                                continue;
                            }

                            for specifier in &import_dclr.specifiers {
                                match specifier {
                                    ImportSpecifier::Named(named_import) => {
                                        // if type only, push it and leave it
                                        if named_import.is_type_only {
                                            let new_node = ModuleItem::ModuleDecl(
                                                ModuleDecl::Import(ImportDecl {
                                                    span: import_dclr.span,
                                                    specifiers: vec![ImportSpecifier::Named(
                                                        ImportNamedSpecifier {
                                                            span: named_import.span,
                                                            is_type_only: false,
                                                            local: named_import.local.clone(),
                                                            imported: named_import.imported.clone(),
                                                        },
                                                    )],
                                                    phase: Default::default(),
                                                    type_only: named_import.is_type_only,
                                                    src: import_dclr.src.clone(),
                                                    with: import_dclr.with.clone(),
                                                }),
                                            );
                                            transformed_nodes.push(new_node);
                                            continue;
                                        }

                                        let imported = &named_import.imported;
                                        let import_var = if let Some(import_name) = imported {
                                            match import_name {
                                                ModuleExportName::Str(s) => {
                                                    debug!("module export name string {:?}", s);
                                                    s.value.to_string()
                                                }
                                                ModuleExportName::Ident(id) => {
                                                    debug!("module export name ident {:?}", id);
                                                    id.sym.to_string()
                                                }
                                            }
                                        } else {
                                            named_import.local.sym.to_string()
                                        };

                                        let transformed_path = self.transform_import_path(
                                            &config.transform,
                                            &self.convert_member_case(
                                                &config.transform_case,
                                                &import_var,
                                            ),
                                            &import_dclr.src,
                                        );

                                        let new_specifier = if config.keep_import_specifier {
                                            specifier.clone()
                                        } else {
                                            ImportSpecifier::Default(ImportDefaultSpecifier {
                                                local: named_import.local.clone(),
                                                span: named_import.span,
                                            })
                                        };

                                        let new_node = ModuleItem::ModuleDecl(ModuleDecl::Import(
                                            ImportDecl {
                                                span: import_dclr.span,
                                                specifiers: vec![new_specifier],
                                                type_only: import_dclr.type_only,
                                                src: Box::new(transformed_path),
                                                with: import_dclr.with.clone(),
                                                phase: Default::default(),
                                            },
                                        ));

                                        transformed_nodes.push(new_node);

                                        if let Some(style) = &config.style_path {
                                            let style_path = self.transform_import_path(
                                                style,
                                                &self.convert_member_case(
                                                    &config.transform_case,
                                                    &import_var,
                                                ),
                                                &import_dclr.src,
                                            );
                                            let new_node = ModuleItem::ModuleDecl(
                                                ModuleDecl::Import(ImportDecl {
                                                    span: import_dclr.span,
                                                    specifiers: vec![],
                                                    type_only: import_dclr.type_only,
                                                    src: Box::new(style_path),
                                                    with: import_dclr.with.clone(),
                                                    phase: Default::default(),
                                                }),
                                            );
                                            transformed_nodes.push(new_node);
                                        }
                                    }
                                    _ => {
                                        let new_node = ModuleItem::ModuleDecl(ModuleDecl::Import(
                                            ImportDecl {
                                                span: import_dclr.span,
                                                specifiers: vec![specifier.clone()],
                                                type_only: import_dclr.type_only,
                                                src: import_dclr.src.clone(),
                                                with: import_dclr.with.clone(),
                                                phase: Default::default(),
                                            },
                                        ));
                                        transformed_nodes.push(new_node);
                                    }
                                }
                            }
                        } else {
                            transformed_nodes.push(node.clone())
                        }
                    }
                    _ => transformed_nodes.push(node.clone()),
                },
                n => transformed_nodes.push(n.clone()),
            }
        }

        nodes.clear();
        nodes.extend(transformed_nodes)
    }
}
