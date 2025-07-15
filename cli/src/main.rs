use doggo_core::manifest::Manifest;

fn main() {
    let manifest = Manifest::load("./");

    println!("{:#?}", manifest);
}
