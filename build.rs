fn main() {
    // Generate uniffi bindings if uniffi feature is enabled
    #[cfg(feature = "uniffi")]
    {
        uniffi::generate_scaffolding("src/libreconomy.udl").unwrap();
    }

    // Regenerate bindings when UDL changes
    println!("cargo:rerun-if-changed=src/libreconomy.udl");
}
