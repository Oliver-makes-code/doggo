use doggo_core::{compiler_backend::{ClangCompilerBackend, ExtraCompileOptions}, manifest::Manifest};

fn main() {
    let manifest = Manifest::load("./");

    println!("{:#?}", manifest);

    let backend = ClangCompilerBackend::new().unwrap();

    backend.compile_object("./test_project/src/main.c", "./out.o", &[], &[], &ExtraCompileOptions::default(), false).unwrap();
}
