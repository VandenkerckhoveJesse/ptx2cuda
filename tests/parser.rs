use ptx_syntax::parse_ptx;
use ptx_syntax::r#type::{AddressSize, ModuleDirective, ModuleInfoDirectiveKind, TargetString};

#[test]
fn test_parser_noop() {
    let source = include_str!("./ptx/noop.ptx");
    let module = parse_ptx(source).expect("noop.ptx should parse");

    for directive in module.directives {
        match directive {
            ModuleDirective::ModuleInfo {
                directive:
                    ModuleInfoDirectiveKind::Version {
                        directive: value,
                        span: _,
                    },
                span: _,
            } => {
                let actual_version = Some((value.major, value.minor));
                assert_eq!(actual_version, Some((9, 0)));
            }
            ModuleDirective::ModuleInfo {
                directive:
                    ModuleInfoDirectiveKind::Target {
                        directive: value,
                        span: _,
                    },
                span: _,
            } => {
                let actual_target = value.entries;
                assert!(matches!(
                    actual_target.as_slice(),
                    [TargetString::Sm86 { span: _ }]
                ));
            }
            ModuleDirective::ModuleInfo {
                directive:
                    ModuleInfoDirectiveKind::AddressSize {
                        directive: value,
                        span: _,
                    },
                span: _,
            } => {
                let actual_address_size = value.size;
                assert!(matches!(
                    actual_address_size,
                    AddressSize::Size64 { span: _ }
                ));
            }
            _ => {}
        }
    }
}

#[test]
fn test_parser_invalid() {
    let source = include_str!("./ptx/invalid.ptx");

    assert!(parse_ptx(source).is_err());
}
