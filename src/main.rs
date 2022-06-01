use minimal_elf::write_elf;

fn main() -> std::io::Result<()> {
    write_elf("tiny")?;
    Ok(())
}
