use windres::Build;

fn main() {
    Build::new().compile("silentkeys-resource.rc").unwrap();
}