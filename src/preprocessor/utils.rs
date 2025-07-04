use std::collections::HashMap;

use crate::parser::ast::Expression;

pub fn get_default_definitions() -> HashMap<String, Expression> {
    let mut definitions = HashMap::new();

    match std::env::consts::ARCH {
        "x86" => definitions.insert("__X86__".into(), Expression::StringLiteral("".into())),
        "x86_64" => definitions.insert("__X86_64__".into(), Expression::StringLiteral("".into())),
        "arm" => definitions.insert("__ARM__".into(), Expression::StringLiteral("".into())),
        "aarch64" => definitions.insert("__AARCH64__".into(), Expression::StringLiteral("".into())),
        "m68k" => definitions.insert("__M68K__".into(), Expression::StringLiteral("".into())),
        "mips" => definitions.insert("__MIPS__".into(), Expression::StringLiteral("".into())),
        "mips32r6" => {
            definitions.insert("__MIPS32R6__".into(), Expression::StringLiteral("".into()))
        }
        "mips64" => definitions.insert("__MIPS64__".into(), Expression::StringLiteral("".into())),
        "mips64r6" => {
            definitions.insert("__MIPS64R6__".into(), Expression::StringLiteral("".into()))
        }
        "csky" => definitions.insert("__CSKY__".into(), Expression::StringLiteral("".into())),
        "powerpc" => definitions.insert("__POWERPC__".into(), Expression::StringLiteral("".into())),
        "powerpc64" => {
            definitions.insert("__POWERPC64__".into(), Expression::StringLiteral("".into()))
        }
        "riscv32" => definitions.insert("__RISCV32__".into(), Expression::StringLiteral("".into())),
        "riscv64" => definitions.insert("__RISCV64__".into(), Expression::StringLiteral("".into())),
        "s390x" => definitions.insert("__S390X__".into(), Expression::StringLiteral("".into())),
        "sparc" => definitions.insert("__SPARC__".into(), Expression::StringLiteral("".into())),
        "sparc64" => definitions.insert("__SPARC64__".into(), Expression::StringLiteral("".into())),
        "hexagon" => definitions.insert("__HEXAGON__".into(), Expression::StringLiteral("".into())),
        "loongarch64" => definitions.insert(
            "__LOONGARCH64__".into(),
            Expression::StringLiteral("".into()),
        ),
        _ => unreachable!(),
    };

    match std::env::consts::OS {
        "linux" => definitions.insert("__LINUX__".into(), Expression::StringLiteral("".into())),
        "windows" => definitions.insert("__WINDOWS__".into(), Expression::StringLiteral("".into())),
        "macos" => definitions.insert("__MACOS__".into(), Expression::StringLiteral("".into())),
        "android" => definitions.insert("__ANDROID__".into(), Expression::StringLiteral("".into())),
        "ios" => definitions.insert("__IOS__".into(), Expression::StringLiteral("".into())),
        "openbsd" => definitions.insert("__OPENBSD__".into(), Expression::StringLiteral("".into())),
        "freebsd" => definitions.insert("__FREEBSD__".into(), Expression::StringLiteral("".into())),
        "netbsd" => definitions.insert("__NETBSD__".into(), Expression::StringLiteral("".into())),
        "wasi" => definitions.insert("__WASI__".into(), Expression::StringLiteral("".into())),
        "hermit" => definitions.insert("__HERMIT__".into(), Expression::StringLiteral("".into())),
        "aix" => definitions.insert("__AIX__".into(), Expression::StringLiteral("".into())),
        "apple" => definitions.insert("__APPLE__".into(), Expression::StringLiteral("".into())),
        "dragonfly" => {
            definitions.insert("__DRAGONFLY__".into(), Expression::StringLiteral("".into()))
        }
        "emscripten" => definitions.insert(
            "__EMSCRIPTEN__".into(),
            Expression::StringLiteral("".into()),
        ),
        "espidf" => definitions.insert("__ESPIDF__".into(), Expression::StringLiteral("".into())),
        "fortanix" => {
            definitions.insert("__FORTANIX__".into(), Expression::StringLiteral("".into()))
        }
        "uefi" => definitions.insert("__UEFI__".into(), Expression::StringLiteral("".into())),
        "fuchsia" => definitions.insert("__FUCHSIA__".into(), Expression::StringLiteral("".into())),
        "haiku" => definitions.insert("__HAIKU__".into(), Expression::StringLiteral("".into())),
        "watchos" => definitions.insert("__WATCHOS__".into(), Expression::StringLiteral("".into())),
        "visionos" => {
            definitions.insert("__VISIONOS__".into(), Expression::StringLiteral("".into()))
        }
        "tvos" => definitions.insert("__TVOS__".into(), Expression::StringLiteral("".into())),
        "horizon" => definitions.insert("__HORIZON__".into(), Expression::StringLiteral("".into())),
        "hurd" => definitions.insert("__HURD__".into(), Expression::StringLiteral("".into())),
        "illumos" => definitions.insert("__ILLUMOS__".into(), Expression::StringLiteral("".into())),
        "l4re" => definitions.insert("__L4RE__".into(), Expression::StringLiteral("".into())),
        "nto" => definitions.insert("__NTO__".into(), Expression::StringLiteral("".into())),
        "redox" => definitions.insert("__REDOX__".into(), Expression::StringLiteral("".into())),
        "solaris" => definitions.insert("__SOLARIS__".into(), Expression::StringLiteral("".into())),
        "solid_asp3" => definitions.insert(
            "__SOLID_ASP3__".into(),
            Expression::StringLiteral("".into()),
        ),
        "vita" => definitions.insert("__VITA__".into(), Expression::StringLiteral("".into())),
        "vxworks" => definitions.insert("__VXWORKS__".into(), Expression::StringLiteral("".into())),
        "xous" => definitions.insert("__XOUS__".into(), Expression::StringLiteral("".into())),
        _ => unreachable!(),
    };

    definitions
}
