use spirv_builder::SpirvBuilder;

fn main() {
    let _ = SpirvBuilder::new("./shaders", "spirv-unknown-vulkan1.1")
        .capability(spirv_builder::Capability::Int8)
        .print_metadata(false)
        .build();
}