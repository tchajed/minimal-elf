use iced_x86::code_asm::IcedError;
use minimal_elf::{write_elf, VADDR};

fn create_program() -> Vec<u8> {
    let f = || -> Result<_, IcedError> {
        use iced_x86::code_asm::*;
        let mut a = CodeAssembler::new(64)?;
        // push + pop is 2+1 bytes, which is slightly shorter than even mov(eax, 60)
        a.push(60)?;
        a.pop(rax)?;
        // a.mov(eax, 60)?;
        // zero edi in two bytes
        a.xor(edi, edi)?;
        a.syscall()?;
        let bytes = a.assemble(VADDR)?;
        Ok(bytes)
    };
    f().unwrap()
}

#[cfg(test)]
mod tests {
    use super::create_program;

    #[test]
    fn test_create_program() {
        let program = create_program();
        assert_eq!(7, program.len());
    }
}

fn main() -> std::io::Result<()> {
    write_elf(&create_program(), "tiny")?;
    Ok(())
}
