#![allow(non_camel_case_types)]
use binary_layout::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use iced_x86::code_asm::IcedError;

// ELF-64 standard from https://uclibc.org/docs/elf-64-gen.pdf and
// https://uclibc.org/docs/psABI-x86_64.pdf (for the value EM_X86_64, specific to x86-64).

type Elf64_Addr = u64;
type Elf64_Off = u64;
type Elf64_Half = u16;
type Elf64_Word = u32;
// type Elf64_Sword = i32;
type Elf64_Xword = u64;
// type Elf64_Sxword = i64;

define_layout!(elf64_ident, LittleEndian, {
    mag: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    os_abi: u8,
    abi_version: u8,
    pad: [u8; 7],
});

#[cfg(test)]
mod ident_tests {
    use super::elf64_ident;

    #[test]
    fn ident_size_ok() {
        // XXX: could be a static assertion but Option<>.unwrap() is not a const_fn
        assert_eq!(16, elf64_ident::SIZE.unwrap());
    }
}

fn set_ident<S: AsRef<[u8]> + AsMut<[u8]>>(mut view: elf64_ident::View<S>) {
    view.mag_mut()
        .write(&[0x7f, 'E' as u8, 'L' as u8, 'F' as u8])
        .unwrap();
    view.class_mut().write(2); // class: ELFCLASS64
    view.data_mut().write(1); // data encoding: ELFDATA2LSB
    view.version_mut().write(1); // file version: EV_CURRENT
    view.os_abi_mut().write(0); // OS/ABI identification: System V
    view.abi_version_mut().write(0); // ABI version: System V third edition
    view.pad_mut().copy_from_slice(&[0u8; 7]);
}

define_layout!(elf64_hdr, LittleEndian, {
    ident: elf64_ident::NestedView,
    _type: Elf64_Half,
    machine: Elf64_Half,
    version: Elf64_Word,
    entry: Elf64_Addr, // virtual address of entry point
    phoff: Elf64_Off, // program header
    shoff: Elf64_Off, // section header
    flags: Elf64_Word, // processor-specific
    ehsize: Elf64_Half,
    phentsize: Elf64_Half,
    phnum: Elf64_Half, // number of program header entries
    shentsize: Elf64_Half, // size of section header entry
    shnum: Elf64_Half, // number of section header entries
    shstrndx: Elf64_Half, // section name string table index
});

fn set_elf64_hdr<S: AsRef<[u8]> + AsMut<[u8]>>(mut view: elf64_hdr::View<S>) {
    set_ident(view.ident_mut());
    view._type_mut().write(2); // ET_EXEC
    view.machine_mut().write(62); // EM_X86_64
    view.version_mut().write(1); // EV_CURRENT
    view.entry_mut().write(VADDR + program_offset());
    view.phoff_mut().write(elf64_hdr::SIZE.unwrap() as u64);
    view.flags_mut().write(0); // no processor-specific flags
    view.ehsize_mut().write(elf64_hdr::SIZE.unwrap() as u16);
    view.phentsize_mut().write(elf64_phdr::SIZE.unwrap() as u16);
    view.phnum_mut().write(1);
}

define_layout!(elf64_phdr, LittleEndian, {
    _type: Elf64_Word,
    flags: Elf64_Word,
    offset: Elf64_Off,
    vaddr: Elf64_Addr,
    paddr: Elf64_Addr,
    filesz: Elf64_Xword,
    memsz: Elf64_Xword,
    align: Elf64_Xword,
});

define_layout!(elf64_file, LittleEndian, {
    hdr: elf64_hdr::NestedView,
    phdr: elf64_phdr::NestedView,
    program: [u8],
});

fn try_create_program() -> Result<Vec<u8>, IcedError> {
    use iced_x86::code_asm::*;
    let mut a = CodeAssembler::new(64)?;
    // push + pop is 2+1 bytes, which is slightly shorter than even mov(eax, 60)
    a.push(60)?;
    a.pop(rax)?;
    // a.mov(eax, 60)?;
    // zero eax in two bytes (unnecessary because Linux zeroes all registers)
    // a.xor(eax, eax)?;
    a.syscall()?;
    let bytes = a.assemble(VADDR)?;
    Ok(bytes)
}

fn create_program() -> Vec<u8> {
    try_create_program().unwrap()
}

#[cfg(test)]
mod program_tests {
    use super::create_program;

    #[test]
    fn test_create_program() {
        let program = create_program();
        assert_eq!(5, program.len());
    }
}

fn program_offset() -> u64 {
    (elf64_hdr::SIZE.unwrap() + elf64_phdr::SIZE.unwrap()) as u64
}
const VADDR: u64 = 0x400000;

fn set_elf64_phdr<S>(mut view: elf64_phdr::View<S>, program_size: u64)
where
    S: AsRef<[u8]> + AsMut<[u8]>,
{
    view._type_mut().write(1); // PT_LOAD
    view.flags_mut().write(0x1 | 0x2 | 0x4); // PF_X | PF_W | PF_R

    // location of segment in file
    view.offset_mut().write(program_offset());
    // virtual address of segment
    view.vaddr_mut().write(VADDR + program_offset());

    view.filesz_mut().write(program_size);
    view.memsz_mut().write(program_size);
    view.align_mut().write(4096);
}

fn main() -> std::io::Result<()> {
    let program = create_program();
    let hdr_sz = elf64_hdr::SIZE.unwrap();
    let phdr_sz = elf64_phdr::SIZE.unwrap();
    let mut buf = vec![0u8; hdr_sz + phdr_sz + program.len()];
    let mut file = elf64_file::View::new(&mut buf);
    set_elf64_hdr(file.hdr_mut());
    set_elf64_phdr(file.phdr_mut(), program.len() as u64);
    file.program_mut().copy_from_slice(&program);

    let mut options = OpenOptions::new();
    options.write(true).create(true).mode(0o755);
    let mut file = options.open("tiny")?;
    file.write_all(&buf)?;
    Ok(())
}
