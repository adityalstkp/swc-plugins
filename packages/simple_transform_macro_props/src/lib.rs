use serde::Deserialize;
use swc_common::DUMMY_SP;
use swc_core::{alloc::collections::FxHashMap, common::util::take::Take};
use swc_ecma_ast::{
    CallExpr, Callee, Expr, Id, Ident, ImportDecl, ImportSpecifier, MemberProp, Program, TaggedTpl,
};
use swc_ecma_utils::ExprFactory;
use swc_ecma_visit::{Fold, FoldWith};
use swc_plugin_macro::plugin_transform;
use swc_plugin_proxy::TransformPluginProgramMetadata;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleTransformPluginOptions {
    pub packages: Vec<String>,
}

#[plugin_transform]
fn simple_transform_macro_props_plugin(
    program: Program,
    data: TransformPluginProgramMetadata,
) -> Program {
    let config = serde_json::from_str::<SimpleTransformPluginOptions>(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .expect("invalid config for emotion");

    program.fold_with(&mut simple_transform_macro_props(config))
}

pub fn simple_transform_macro_props(options: SimpleTransformPluginOptions) -> impl Fold {
    MacroTransformer::new(options)
}

struct MacroTransformer {
    import_packages: FxHashMap<Id, bool>,
    registered_packages: Vec<String>,
}

impl MacroTransformer {
    pub fn new(options: SimpleTransformPluginOptions) -> Self {
        MacroTransformer {
            registered_packages: options.packages,
            import_packages: FxHashMap::default(),
        }
    }

    fn generate_import_info(&mut self, expr: &ImportDecl) {
        for pkg in self.registered_packages.iter() {
            if expr.src.value == *pkg {
                for specifier in expr.specifiers.iter() {
                    match specifier {
                        ImportSpecifier::Default(default) => {
                            self.import_packages.insert(default.local.to_id(), true);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Fold for MacroTransformer {
    fn fold_import_decl(&mut self, expr: ImportDecl) -> ImportDecl {
        if expr.type_only {
            return expr;
        }
        self.generate_import_info(&expr);
        expr
    }

    fn fold_call_expr(&mut self, mut expr: CallExpr) -> CallExpr {
        if let Callee::Expr(e) = &mut expr.callee {
            match e.as_mut() {
                Expr::Member(m) => {
                    if let Expr::Ident(i) = m.obj.as_ref() {
                        if let Some(p) = self.import_packages.get(&i.to_id()) {
                            if *p {
                                if let MemberProp::Ident(prop) = &mut m.prop {
                                    return CallExpr {
                                        span: expr.span,
                                        args: expr.args,
                                        callee: CallExpr {
                                            span: DUMMY_SP,
                                            callee: Ident::new(i.sym.clone(), i.span, i.ctxt)
                                                .as_callee(),
                                            args: vec![prop.take().sym.as_arg()],
                                            ..Default::default()
                                        }
                                        .as_callee(),
                                        ..Default::default()
                                    };
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        expr
    }

    fn fold_expr(&mut self, mut expr: Expr) -> Expr {
        if let Expr::TaggedTpl(tagged_tpl) = &mut expr {
            match tagged_tpl.tag.as_mut() {
                Expr::Member(member_expr) => {
                    if let Expr::Ident(i) = member_expr.obj.as_mut() {
                        if let Some(p) = self.import_packages.get(&i.to_id()) {
                            if let MemberProp::Ident(prop) = &mut member_expr.prop {
                                if *p {
                                    let tpl = &tagged_tpl.tpl;
                                    return Expr::TaggedTpl(TaggedTpl {
                                        tag: Box::new(Expr::Call(CallExpr {
                                            callee: i.take().as_callee(),
                                            args: vec![prop.take().sym.as_arg()],
                                            ..Default::default()
                                        })),
                                        tpl: tpl.clone(),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        expr.fold_children_with(self)
    }
}
