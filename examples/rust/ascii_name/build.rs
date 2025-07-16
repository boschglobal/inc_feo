use anyhow::Result;
use vergen::{BuildBuilder, Emitter};

pub fn main() -> Result<()> {
    let build = BuildBuilder::all_build()?;
    Emitter::default().add_instructions(&build)?.emit()
}
