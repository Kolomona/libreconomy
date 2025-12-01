fn main() {
    // Generate uniffi bindings if needed
    println!("cargo:rerun-if-changed=src/libreconomy.udl");
}
