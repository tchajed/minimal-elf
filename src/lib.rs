//! Create a minimal ELF file manually.
//!
//! Based on reading the [ELF-64
//! standard](https://uclibc.org/docs/elf-64-gen.pdf) and the [x86-64
//! architecture supplement](https://uclibc.org/docs/psABI-x86_64.pdf) (for the
//! value `EM_X86_64`, specific to x86-64).
//!
//! Also learned from the classic blog post "[A Whirlwind Tutorial on Creating
//! Really Teensy Elf Executables for
//! Linux](https://www.muppetlabs.com/~breadbox/software/tiny/teensy.html)" and
//! the code in <https://github.com/AjayBrahmakshatriya/minimal-elf/>.

#![allow(non_camel_case_types)]

use binary_layout::prelude::*;
use std::io::prelude::*;
use std::path::Path;
use std::{fs::OpenOptions, os::unix::prelude::OpenOptionsExt};

type Elf64_Addr = u64;
type Elf64_Off = u64;
type Elf64_Half = u16;
type Elf64_Word = u32;
// type Elf64_Sword = i32;
type Elf64_Xword = u64;
// type Elf64_Sxword = i64;

binary_layout!(elf64_ident, LittleEndian, {
    mag: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    os_abi: u8,
    abi_version: u8,
    pad: [u8; 7],
});

#[cfg(test)]
mod tests {
    use super::elf64_ident;

    #[test]
    fn ident_size_ok() {
        // XXX: could be a static assertion but Option<>::unwrap() is not a
        // const_fn
        assert_eq!(16, elf64_ident::SIZE.unwrap());
    }
}

fn set_ident<S: AsRef<[u8]> + AsMut<[u8]>>(mut view: elf64_ident::View<S>) {
    view.mag_mut()
        .copy_from_slice(&[0x7f, 'E' as u8, 'L' as u8, 'F' as u8]);
    view.class_mut().write(2); // class: ELFCLASS64
    view.data_mut().write(1); // data encoding: ELFDATA2LSB
    view.version_mut().write(1); // file version: EV_CURRENT
    view.os_abi_mut().write(0); // OS/ABI identification: System V
    view.abi_version_mut().write(0); // ABI version: System V third edition
    view.pad_mut().copy_from_slice(&[0u8; 7]);
}

binary_layout!(elf64_hdr, LittleEndian, {
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

const PROGRAM_OFFSET: u64 = {
    // XXX: manually implement unwrap since it isn't stable as a const fn
    let sz1 = match elf64_hdr::SIZE {
        Some(s) => s,
        None => panic!("unsized"),
    };
    let sz2 = match elf64_phdr::SIZE {
        Some(s) => s,
        None => panic!("unsized"),
    };
    (sz1 + sz2) as u64
};

pub const VADDR: u64 = 0x400000;

fn set_elf64_hdr<S: AsRef<[u8]> + AsMut<[u8]>>(mut view: elf64_hdr::View<S>) {
    set_ident(view.ident_mut());
    view._type_mut().write(2); // ET_EXEC
    view.machine_mut().write(62); // EM_X86_64
    view.version_mut().write(1); // EV_CURRENT
    view.entry_mut().write(VADDR + PROGRAM_OFFSET);
    view.phoff_mut().write(elf64_hdr::SIZE.unwrap() as u64);
    view.flags_mut().write(0); // no processor-specific flags
    view.ehsize_mut().write(elf64_hdr::SIZE.unwrap() as u16);
    view.phentsize_mut().write(elf64_phdr::SIZE.unwrap() as u16);
    view.phnum_mut().write(1);
}

binary_layout!(elf64_phdr, LittleEndian, {
    _type: Elf64_Word,
    flags: Elf64_Word,
    offset: Elf64_Off,
    vaddr: Elf64_Addr,
    paddr: Elf64_Addr,
    filesz: Elf64_Xword,
    memsz: Elf64_Xword,
    align: Elf64_Xword,
});

fn set_elf64_phdr<S>(mut view: elf64_phdr::View<S>, program_size: u64)
where
    S: AsRef<[u8]> + AsMut<[u8]>,
{
    view._type_mut().write(1); // PT_LOAD
    view.flags_mut().write(0x1 | 0x2 | 0x4); // PF_X | PF_W | PF_R

    // location of segment in file
    let offset = (elf64_hdr::SIZE.unwrap() + elf64_phdr::SIZE.unwrap()) as u64;
    view.offset_mut().write(offset);
    // virtual address of segment
    view.vaddr_mut().write(VADDR + PROGRAM_OFFSET);

    view.filesz_mut().write(program_size);
    view.memsz_mut().write(program_size);
    view.align_mut().write(4096);
}

binary_layout!(elf64_file, LittleEndian, {
    hdr: elf64_hdr::NestedView,
    phdr: elf64_phdr::NestedView,
    program: [u8],
});

fn create_elf(program: &[u8]) -> Vec<u8> {
    let hdr_sz = elf64_hdr::SIZE.unwrap();
    let phdr_sz = elf64_phdr::SIZE.unwrap();
    let mut buf = vec![0u8; hdr_sz + phdr_sz + program.len()];
    let mut file = elf64_file::View::new(&mut buf);
    set_elf64_hdr(file.hdr_mut());
    set_elf64_phdr(file.phdr_mut(), program.len() as u64);
    file.program_mut().copy_from_slice(program);
    buf
}

fn create_program() -> Vec<u8> {
    use iced_x86::code_asm::*;
    let f = || -> Result<_, IcedError> {
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
mod test_program {
    use super::create_program;

    #[test]
    fn test_create_program() {
        let program = create_program();
        assert_eq!(7, program.len());
    }
}

pub fn write_elf<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let buf = create_elf(&create_program());
    let mut options = OpenOptions::new();
    options.write(true).create(true).mode(0o755);
    let mut file = options.open(path)?;
    file.write_all(&buf)?;
    Ok(())
}
